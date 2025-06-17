pub mod wrapper;
use std::sync::mpsc;
use self::wrapper::{RUdpClient, RUdpServer};
use crate::{
    protocol::{tcp::TcpConnection, TUNNEL_FLAG},
    DrakulaProtocol,
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use crossbeam_channel::{bounded, Sender, Receiver};

use super::{Client, Server};

pub struct UDPServer {
    local_addr: SocketAddr,
    closed: Arc<AtomicBool>,
    connections: Arc<Mutex<HashMap<SocketAddr, Arc<RUdpClient>>>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

pub struct UDPConnection {
    s: Option<RUdpClient>,
    local_addr: SocketAddr,
    closed: Arc<AtomicBool>,
}

impl Server for UDPServer {
    fn new<
        CBCB: 'static + Fn(crate::packet::Message) + Send + Copy,
        CB: 'static + Fn(crate::DrakulaProtocol, Vec<u8>, SocketAddr, CBCB) + Send,
    >(
        address: &str,
        cb_data: CB,
        cbcb: CBCB,
    ) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut server = RUdpServer::new(&address.to_string())?;
        let local_addr = server.local_addr().unwrap();

        let connections: Arc<Mutex<HashMap<SocketAddr, Arc<RUdpClient>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let closed = Arc::new(AtomicBool::new(false));
        let cb_data = Arc::new(Mutex::new(cb_data));

        // Создаем канал для управления остановкой сервера
        let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel();

        let connections_1 = connections.clone();
        let closed_1 = closed.clone();
        let local_addr_1 = local_addr;

        std::thread::Builder::new()
            .name(format!("udp main worker: {}", local_addr_1))
            .spawn(move || {
                let mut threads = Vec::new();

                loop {
                    // Проверяем сигнал остановки
                    if shutdown_rx.try_recv().is_ok() || closed_1.load(Ordering::Relaxed) {
                        break;
                    }

                    let client = match server.accept(500) {
                        Ok(p) => p,
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::TimedOut {
                                std::thread::sleep(std::time::Duration::from_millis(200));
                                continue;
                            } else {
                                closed_1.store(true, Ordering::Relaxed);
                                break;
                            }
                        }
                    };

                    let client = Arc::new(client);
                    let remote_addr = match client.peer_addr() {
                        Ok(addr) => addr,
                        Err(e) => {
                            log::error!("Failed to get peer address: {}", e);
                            continue;
                        }
                    };

                    // Добавляем клиента в connections
                    connections_1.lock().unwrap().insert(remote_addr, client.clone());

                    let connections_2 = connections_1.clone();
                    let cb_data = cb_data.clone();
                    let cbcb = cbcb;
                    let local_addr_str = local_addr_1.to_string();

                    let handle = std::thread::Builder::new()
                        .name(format!("udp client worker: {}", remote_addr))
                        .spawn(move || {
                            let tunnel_active = Arc::new(AtomicBool::new(false));
                            
                            loop {
                                let buf = match client.recv() {
                                    Ok(p) => p,
                                    Err(e) => {
                                        log::error!("udp client recv error: {}", e);
                                        break;
                                    }
                                };

                                if buf.len() >= TUNNEL_FLAG.len() + 3 && buf[1..5] == TUNNEL_FLAG {
                                    if tunnel_active.load(Ordering::Acquire) {
                                        continue;
                                    }
                                    
                                    tunnel_active.store(true, Ordering::Release);
                                    
                                    let port = u16::from_be_bytes([buf[5], buf[6]]);
                                    let full_addr = format!("127.0.0.1:{}", port);
                                    
                                    let tunnel_client = match TcpConnection::connect(&full_addr) {
                                        Ok(p) => p,
                                        Err(e) => {
                                            log::error!("tunnel connect failed: {}", e);
                                            tunnel_active.store(false, Ordering::Release);
                                            continue;
                                        }
                                    };

                                    let (stop_tx, stop_rx): (Sender<()>, Receiver<()>) = bounded(1);
                                    let stop_rx1 = stop_rx.clone();
                                    let stop_rx2 = stop_rx.clone();
                            

                                    let mut tunnel_client1 = tunnel_client.clone();
                                    let client1 = client.clone();
 
                                    let connections_3 = connections_2.clone();
                                    
                                    let handle1 = std::thread::Builder::new()
                                    .name(format!("udp tunnel worker1: {}", remote_addr))
                                    .spawn(move || {
                                        loop {
                                            // Проверяем сигнал остановки без блокировки
                                            if stop_rx1.try_recv().is_ok() {
                                                break;
                                            }
                        
                                            match tunnel_client1.recv() {
                                                Ok(buf) if !buf.is_empty() => {
                                                    let mut data = vec![0xfe];
                                                    data.extend_from_slice(&buf);
                                                    if client1.send(data).is_err() {
                                                        break;
                                                    }
                                                }
                                                Ok(_) => break, // пустой буфер
                                                Err(_) => break, // ошибка приема
                                            }
                        
                                            // Небольшая пауза для снижения нагрузки на CPU
                                            std::thread::sleep(std::time::Duration::from_micros(100));
                                        }
                                        connections_3.lock().unwrap().remove(&remote_addr);
                                    }).unwrap();
                        

                                    let mut tunnel_client2 = tunnel_client.clone();
                                    let client2 = client.clone();
                                    // let stop_tx2 = stop_tx.clone();
                                    let connections_4 = connections_2.clone();
                                    
                                    // let stop_rx2 = stop_rx.clone();
                                let handle2 = std::thread::Builder::new()
                                    .name(format!("udp tunnel worker2: {}", remote_addr))
                                    .spawn(move || {
                                        loop {
                                            if stop_rx2.try_recv().is_ok() {
                                                break;
                                            }
                        
                                            match client2.recv() {
                                                Ok(mut buf) => {
                                                    if tunnel_client2.send(&mut buf[1..]).is_err() {
                                                        break;
                                                    }
                                                }
                                                Err(_) => break,
                                            }
                        
                                            std::thread::sleep(std::time::Duration::from_micros(100));
                                        }
                                        connections_4.lock().unwrap().remove(&remote_addr);
                                    }).unwrap();
                        
                                // Ожидаем завершения потоков
                                handle1.join().unwrap();
                                handle2.join().unwrap();
                                    break;
                                }

                                // Обычная обработка UDP-пакетов
                                cb_data.lock().unwrap()(
                                    DrakulaProtocol::UDP,
                                    buf[1..].to_vec(),
                                    remote_addr,
                                    cbcb,
                                );
                            }
                            
                            connections_2.lock().unwrap().remove(&remote_addr);
                            log::info!("udp client worker finished for {}", remote_addr);
                        }).unwrap();

                    threads.push(handle);
                }

                // Очистка ресурсов
                let mut conns = connections_1.lock().unwrap();
                for (addr, client) in conns.iter() {
                    if let Err(e) = client.close() {
                        log::warn!("Failed to close client {}: {}", addr, e);
                    }
                }
                conns.clear();

                if let Err(e) = server.close() {
                    log::warn!("Failed to close server: {}", e);
                }

                // Дожидаемся завершения всех клиентских потоков
                for handle in threads {
                    if let Err(e) = handle.join() {
                        log::warn!("Thread join error: {:?}", e);
                    }
                }

                log::info!("udp main worker finished");
            }).unwrap();

        Ok(Self {
            local_addr,
            closed,
            connections,
            shutdown_tx: Some(shutdown_tx),
        })
    }

    fn local_addr(&self) -> std::io::Result<SocketAddr> {
        Ok(self.local_addr)
    }

    fn sendto(&mut self, peer_addr: &SocketAddr, buf: &[u8]) -> std::io::Result<()> {
        match self.connections.lock().unwrap().get_mut(peer_addr) {
            Some(k) => {
                let mut data = vec![0xfe];
                data.append(&mut buf.to_vec());

                match k.send(data) {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Interrupted,
                            format!("udp send msg error : {}", e),
                        ));
                    }
                };
                Ok(())
            }
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "not found client",
            )),
        }
    }

    fn contains_addr(&mut self, peer_addr: &SocketAddr) -> bool {
        self.connections.lock().unwrap().contains_key(peer_addr)
    }

    fn close(&mut self) {
        self.closed
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Drop for UDPServer {
    fn drop(&mut self) {
        self.close();
    }
}

impl Client for UDPConnection {
    fn connect(address: &str) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let s = RUdpClient::new(address.to_string())?;

        let local_addr = s.local_addr().unwrap();

        Ok(Self {
            s: Some(s),
            closed: Arc::new(AtomicBool::new(false)),
            local_addr,
        })
    }

    fn tunnel(remote_addr: &str, server_local_port: u16) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        log::info!("start tunnel [{}] [{}]", remote_addr, server_local_port);
        let s = RUdpClient::new(remote_addr.to_string())?;

        let local_addr = s.local_addr().unwrap();

        let mut buf = TUNNEL_FLAG.to_vec();
        buf.append(&mut server_local_port.to_be_bytes().to_vec());

        let mut ret = Self {
            s: Some(s),
            closed: Arc::new(AtomicBool::new(false)),
            local_addr,
        };

        ret.send(&mut buf)?;

        Ok(ret)
    }

    fn recv(&mut self) -> std::io::Result<Vec<u8>> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "socket closed",
            ));
        }

        let s = match self.s.as_ref() {
            Some(p) => p,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "socket closed",
                ));
            }
        };

        match s.recv() {
            Ok(msg) => Ok(msg[1..].to_vec()),
            Err(e) => {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    format!("udp receive error : {}", e),
                ))
            }
        }
    }

    fn send(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "socket closed",
            ));
        }

        let s = match self.s.as_ref() {
            Some(p) => p,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "socket closed",
                ));
            }
        };

        let mut data = vec![0xfe];
        data.append(&mut buf.to_vec());

        s.send(data.to_vec())?;
        Ok(())
    }

    fn local_addr(&self) -> std::io::Result<SocketAddr> {
        Ok(self.local_addr)
    }

    fn close(&mut self) {
        self.closed.store(true, Ordering::Relaxed);
        self.s = None;
    }
}

impl Clone for UDPConnection {
    fn clone(&self) -> Self {
        Self {
            s: self.s.clone(),
            closed: self.closed.clone(),
            local_addr: self.local_addr,
        }
    }
}

impl Drop for UDPConnection {
    fn drop(&mut self) {
        self.s = None;
    }
}

#[test]
fn test_udp_tunnel() {
    let server = UDPServer::new("127.0.0.1:0", |_, _, _, _| {}, |_| {}).unwrap();
    let server2 = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let remote_local_port = server2.local_addr().unwrap().port();

    let remote = &format!("127.0.0.1:{}", server.local_addr().unwrap().port());
    let mut client1 = UDPConnection::tunnel(remote, remote_local_port).unwrap();

    let (mut client2, _) = super::tcp::TcpConnection::tunnel_server(server2, 10).unwrap();

    for _ in 0..3 {
        client1.send(&mut [0, 1, 2, 3]).unwrap();
        let buf = client2.recv().unwrap();
        assert_eq!(buf, [0, 1, 2, 3]);

        client2.send(&mut [4, 5, 6, 7]).unwrap();
        let buf = client1.recv().unwrap();
        assert_eq!(buf, [4, 5, 6, 7]);
    }
}
