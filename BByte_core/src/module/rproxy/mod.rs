mod rproxy_port;
// mod controller;
// mod model;
use self::rproxy_port::*;
use BByte_util::session::{Session, SessionBase, SessionPacket};
use std::env::current_dir;
use std::sync::{atomic::AtomicBool,    };

mod controller;
use controller::*;


use BByte_util::{
    DrakulaServerCommandID};
mod model;

use model::FileUpload;

use std::{
    collections::HashMap,
    sync::{
        mpsc::{channel, Sender},
        Arc, RwLock,
    },
};
use BByte_util::protocol::tcp::TcpConnection;
use BByte_util::protocol::Client;
use std::net::TcpListener;
use std::sync::mpsc;
use BByte_util::ftp::FTPPacket;

static MAGIC_FLAG: [u8; 2] = [0x37, 0x37];

#[derive(Clone)]
pub struct ServerRproxy {
    id: String,
    clientid: String,
    closed: Arc<AtomicBool>,
    term: TermInstance,
    sender: Sender<SessionBase>,
    proxy_running: bool, 
}
impl ServerRproxy {
    pub fn set_proxy_running(&mut self, value: bool) {
        self.proxy_running = value;
    }
}
impl Session for ServerRproxy {
    fn new(
        sender: Sender<SessionBase>,
        clientid: &String,
        peer_addr: &String,
    ) -> std::io::Result<Self> {
        let closed = Arc::new(AtomicBool::new(false));

        #[cfg(not(target_os = "windows"))]
        let driver_path = current_dir()
            .unwrap()
            .join("heroinn_shell")
            .to_str()
            .unwrap()
            .to_string();

        #[cfg(target_os = "windows")]
        let driver_path = current_dir()
            .unwrap()
            .join("libs\\Resource_proxy.bin")
            .to_str()
            .unwrap()
            .to_string();
        
        log::debug!("sdadas");
        log::info!("sdadas1 INFO");
        let (sender2, receiver) = channel::<FTPPacket>();
        let term = match TermInstance::new(&driver_path, peer_addr,sender2) {
            Ok(p) => p,
            Err(e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                ));
            }
        };

        // let server = TcpListener::bind("127.0.0.1:0")?;
        // let port2 = server.local_addr().unwrap().port();

        // let mut ss2 = TcpConnection::connect(&format!("127.0.0.1:{}", port2)).unwrap();
        // let mut s2 = ss2.clone();
        // // Создаем TCP-сервер для прокси
        // let (sender, receiver) = channel::<FTPPacket>();
        // std::thread::Builder::new()
        // .name("ftp sender worker".to_string())
        // .spawn(move || loop {
        //     let msg = match receiver.recv() {
        //         Ok(p) => p,
        //         Err(_) => {
        //             std::process::exit(0);
        //         }
        //     };
        //     log::debug!("ftp send msg to core : {}", msg.id);
        //     s2.send(&mut msg.serialize().unwrap()).unwrap();
        // })
        // .unwrap();



       


      

        let (senders, receiver) = mpsc::channel::<Vec<u8>>();

        // let mut term_1 = term.clone();
        let closed_2 = closed.clone();
        std::thread::spawn(move || {
            // term_1.wait_for_exit().unwrap();
            // closed_2.store(true, std::sync::atomic::Ordering::Relaxed);
        });
        // let (sender, receiver) = channel::<SessionBase>();
        let id = uuid::Uuid::new_v4().to_string();
        let id_1 = id.clone();
        let closed_1 = closed.clone();
        let clientid_1 = clientid.clone();
        let mut term_2 = term.clone();
        let sender_1 = sender.clone();


        let client_sender = senders.clone();
        std::thread::spawn(move || {


            // let rproxy_message = {
            //     let mut buf = Vec::new();
            //     buf.push(DrakulaServerCommandID::Rproxy.to_u8()); // Первый байт — идентификатор команды
            //     buf.extend_from_slice(b"additisadonal_dataadditisadonal_dataadditisadonal_dataadditisadonal_dataadditisadonal_dataadditisadonal_dataadditisadonal_dataadditisadonal_data"); // Остальные данные (пример)
            //     buf
            // };
            
            // // Отправляем сообщение
            // if let Err(e) = client_sender.send(rproxy_message) {
            //     log::error!("Failed to send message through channel: {}", e);
            // }

            // let mut buf = [0u8; 1024];
            //     let size = match term_2.read(&mut buf) {
            //         Ok(p) => p,
            //         Err(e) => {
            //             log::error!("term instance read error : {}", e);
            //             // break;
            //             0
            //         }
            //     };
            //     if size > 0 {
            //         buf[0] = 5;
            //     }
            //     std::thread::sleep(std::time::Duration::from_secs(3));
            //     let packet = SessionPacket {
            //         id: id_1.clone(),
            //         data: buf[..size].to_vec(),
            //     };

            //     match sender_1.send(SessionBase {
            //         id: id_1.clone(),
            //         clientid: clientid_1.clone(),
            //         packet,
            //     }) {
            //         Ok(_) => {}
            //         Err(e) => {
            //             log::info!("sender closed : {}", e);
            //             // break;
            //         }
            //     };

            // loop {
            //     break;
            //     if closed_1.load(std::sync::atomic::Ordering::Relaxed) {
            //         break;
            //     }

            //     let mut buf = [0u8; 1024];
            //     let size = match term_2.read(&mut buf) {
            //         Ok(p) => p,
            //         Err(e) => {
            //             log::error!("term instance read error : {}", e);
            //             break;
            //         }
            //     };
            //     std::thread::sleep(std::time::Duration::from_secs(10));
            //     let packet = SessionPacket {
            //         id: id_1.clone(),
            //         data: buf[..size].to_vec(),
            //     };

            //     match sender_1.send(SessionBase {
            //         id: id_1.clone(),
            //         clientid: clientid_1.clone(),
            //         packet,
            //     }) {
            //         Ok(_) => {}
            //         Err(e) => {
            //             log::info!("sender closed : {}", e);
            //             break;
            //         }
            //     };
            // }
            log::info!("rprox worker closed");
            closed_1.store(true, std::sync::atomic::Ordering::Relaxed);
        });

        Ok(Self {
            id,
            closed,
            clientid: clientid.clone(),
            term,
            sender,
            proxy_running: false,
        })
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn write(&mut self, data: &Vec<u8>) -> std::io::Result<()> {
        if data.len() == 3 && self.alive() && data == &vec![MAGIC_FLAG[0], MAGIC_FLAG[1], 0xff] {
            log::info!("client closed reverse proxy");
            self.close();
            return Ok(());
        }
        // return Ok(());
        self.term.write(data)
    }

    fn close(&mut self) {
        log::info!("shell closed");

        let packet = SessionPacket {
            id: self.id.clone(),
            data: vec![MAGIC_FLAG[0], MAGIC_FLAG[1], 0xff],
        };

        match self.sender.send(SessionBase {
            id: self.id.clone(),
            clientid: self.clientid.clone(),
            packet,
        }) {
            Ok(_) => {}
            Err(_) => {}
        };

        self.term.close().unwrap();
        self.closed
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    fn alive(&self) -> bool {
        !self.closed.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn clientid(&self) -> String {
        self.clientid.clone()
    }

    fn new_client(
        _sender: Sender<SessionBase>,
        _clientid: &String,
        _id: &String,
    ) -> std::io::Result<ServerRproxy> {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "not client",
        ))
    }
    
}


 