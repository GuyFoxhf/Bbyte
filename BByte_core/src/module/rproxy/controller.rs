// use BByte_util::rot13_in_place;
use BByte_util::{
    cur_timestamp_secs,
    ftp::{
        method::{get_disk_info, get_folder_info, md5_file, remove_file},
        FTPGetHeader, FTPId, FTPPacket, FTPPutHeader, FileInfo,
    },
    packet::TunnelRequest,
    protocol::{tcp::TcpConnection, Client},
    rpc::RpcMessage,
};
use BByte_util::rpc::RpcClient;
use std::collections::HashMap;
use std::sync::RwLock;
// use crate::lib;

use std::{io::*, net::TcpListener, sync::mpsc::Sender};
// crate::module::rproxy::TransferInfo;
// use crate::{TransferInfo, G_RPCCLIENT, G_TRANSFER, FtpApp};
use lazy_static::lazy_static;
// use crate::FTP_APP;
lazy_static! {
    static ref G_RPCCLIENT: Arc<RpcClient> = Arc::new(RpcClient::new());
    static ref G_TRANSFER: Arc<RwLock<HashMap<String, TransferInfo>>> =
        Arc::new(RwLock::new(HashMap::new()));
}
#[derive(Debug, Clone)]
pub struct TransferInfo {
    pub typ: String,
    pub local_path: String,
    pub remote_path: String,
    pub size: f64,
    pub remaind_size: f64,
    pub speed: f64,
    pub remaind_time: f64,
}
use::std::sync::{Mutex};



fn send_ftp_packet(sender: &Sender<FTPPacket>, packet: FTPPacket) -> Result<()> {
    match sender.send(packet) {
        Ok(_) => Ok(()),
        Err(_) => Err(std::io::Error::new(
            std::io::ErrorKind::Interrupted,
            "sender ftp packet faild",
        )),
    }
}

fn build_ftp_rpc_packet(rpc_data: &RpcMessage) -> Result<FTPPacket> {
    Ok(FTPPacket {
        id: FTPId::RPC.to_u8(),
        data: rpc_data.serialize()?,
    })
}

pub fn get_remote_disk_info(sender: &Sender<FTPPacket>) -> Result<Vec<FileInfo>> {
    let msg = RpcMessage::build_call("get_disk_info", vec![]);
    let mut remote_disk_info = vec![];
    send_ftp_packet(sender, build_ftp_rpc_packet(&msg)?)?;
    match G_RPCCLIENT.wait_msg(&msg.id, 10) {
        Ok(p) => {
            if p.retcode != 0 {
                return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, p.msg));
            }

            for i in &p.data {
                let item = FileInfo::parse(i).unwrap();
                remote_disk_info.push(item);
            }

            Ok(remote_disk_info)
        }
        Err(e) => Err(e),
    }
}

pub fn get_local_disk_info() -> Result<Vec<FileInfo>> {
    let mut local_disk_info = vec![];
    match get_disk_info(vec![]) {
        Ok(p) => {
            for i in &p {
                let item = FileInfo::parse(i).unwrap();
                local_disk_info.push(item);
            }

            Ok(local_disk_info)
        }
        Err(e) => Err(e),
    }
}

pub fn get_remote_folder_info(
    sender: &Sender<FTPPacket>,
    full_path: &String,
) -> Result<Vec<FileInfo>> {
    let msg = RpcMessage::build_call("get_folder_info", vec![full_path.clone()]);
    let mut remote_folder_info = vec![];
    send_ftp_packet(sender, build_ftp_rpc_packet(&msg)?)?;
    match G_RPCCLIENT.wait_msg(&msg.id, 10) {
        Ok(p) => {
            if p.retcode != 0 {
                return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, p.msg));
            }

            for i in &p.data {
                let item = FileInfo::parse(i).unwrap();
                remote_folder_info.push(item);
            }

            Ok(remote_folder_info)
        }
        Err(e) => Err(e),
    }
}

pub fn get_remote_join_path(
    sender: &Sender<FTPPacket>,
    cur_path: &String,
    filename: &String,
) -> Result<String> {
    let msg = RpcMessage::build_call("join_path", vec![cur_path.clone(), filename.clone()]);
    send_ftp_packet(sender, build_ftp_rpc_packet(&msg)?)?;
    match G_RPCCLIENT.wait_msg(&msg.id, 10) {
        Ok(p) => {
            if p.retcode != 0 {
                return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, p.msg));
            }

            Ok(p.data[0].clone())
        }
        Err(e) => Err(e),
    }
}

pub fn get_local_folder_info(full_path: &String) -> Result<Vec<FileInfo>> {
    let mut local_folder_info = vec![];
    match get_folder_info(vec![full_path.clone()]) {
        Ok(p) => {
            for i in &p {
                let item = FileInfo::parse(i).unwrap();
                local_folder_info.push(item);
            }

            Ok(local_folder_info)
        }
        Err(e) => Err(e),
    }
}

pub fn delete_local_file(full_path: &String) -> Result<()> {
    remove_file(vec![full_path.clone()])?;
    Ok(())
}

pub fn delete_remote_file(sender: &Sender<FTPPacket>, full_path: &String) -> Result<()> {
    let msg = RpcMessage::build_call("remove_file", vec![full_path.clone()]);
    send_ftp_packet(sender, build_ftp_rpc_packet(&msg)?)?;
    match G_RPCCLIENT.wait_msg(&msg.id, 10) {
        Ok(p) => {
            if p.retcode != 0 {
                return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, p.msg));
            }

            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub fn download_file(
    sender: &Sender<FTPPacket>,
    local_path: &String,
    remote_path: &String,
) -> Result<()> {
    let local_md5 = match md5_file(vec![local_path.to_string()]) {
        Ok(p) => p[0].clone(),
        Err(_) => String::new(),
    };

    let mut header = FTPGetHeader {
        path: remote_path.clone(),
        start_pos: 0,
    };

    let (mut f, total_size) = if !local_md5.is_empty() {
        let mut f = std::fs::File::options().write(true).open(local_path)?;

        let msg = RpcMessage::build_call(
            "md5_file",
            vec![remote_path.clone(), f.metadata()?.len().to_string()],
        );
        send_ftp_packet(sender, build_ftp_rpc_packet(&msg)?)?;
        let (remote_md5, file_size) = match G_RPCCLIENT.wait_msg(&msg.id, 10) {
            Ok(p) => {
                if p.retcode != 0 {
                    return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, p.msg));
                }

                (p.data[0].clone(), p.data[1].parse::<u64>().unwrap())
            }
            Err(e) => {
                return Err(e);
            }
        };

        if remote_md5 == local_md5 {
            log::info!("resume broken transfer");
            header.start_pos = f.metadata()?.len();

            match f.seek(SeekFrom::Start(header.start_pos)) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("seek local file faild : {}", e);
                    return Err(e);
                }
            };
        }

        (f, file_size)
    } else {
        let f = std::fs::File::create(local_path)?;

        let msg = RpcMessage::build_call("file_size", vec![remote_path.clone()]);
        send_ftp_packet(sender, build_ftp_rpc_packet(&msg)?)?;
        let file_size = match G_RPCCLIENT.wait_msg(&msg.id, 10) {
            Ok(p) => {
                if p.retcode != 0 {
                    return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, p.msg));
                }

                p.data[0].parse::<u64>().unwrap()
            }
            Err(e) => {
                return Err(e);
            }
        };

        (f, file_size)
    };

    let sender = sender.clone();
    let local_path = local_path.clone();
    let remote_path = remote_path.clone();
    std::thread::Builder::new()
        .name("download file worker".to_string())
        .spawn(move || {
            let server = TcpListener::bind("127.0.0.1:0").unwrap();

            let req = TunnelRequest {
                port: server.local_addr().unwrap().port(),
            };

            match sender.send(FTPPacket {
                id: FTPId::Get.to_u8(),
                data: req.serialize().unwrap(),
            }) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("send open tunnel msg faild : {}", e);
                    return;
                }
            }

            let (mut s, _) = match TcpConnection::tunnel_server(server, 10) {
                Ok(p) => p,
                Err(e) => {
                    log::error!("create tunnel server faild : {}", e);
                    return;
                }
            };

            match s.send(&mut header.serialize().unwrap()) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("send get header faild : {}", e);
                    return;
                }
            };

            // init transfer log
            {
                let mut transfer = G_TRANSFER.write().unwrap();
                transfer.insert(
                    local_path.clone(),
                    TransferInfo {
                        typ: "Download".to_string(),
                        local_path: local_path.clone(),
                        remote_path: remote_path.clone(),
                        size: total_size as f64,
                        remaind_size: (total_size - header.start_pos) as f64,
                        speed: 0.0,
                        remaind_time: 999999.0,
                    },
                );
            }

            let mut tick_time = cur_timestamp_secs();

            log::debug!("start get transfer [{}]", header.path);
            loop {
                let data = match s.recv() {
                    Ok(p) => p,
                    Err(e) => {
                        log::error!("recv data faild from ftp slave : {}", e);
                        break;
                    }
                };

                if data.is_empty() {
                    break;
                }

                match f.write_all(&data) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("write download file faild : {}", e);
                        break;
                    }
                };
                log::debug!("recv transfer data [{}]", data.len());
                let pos = match f.stream_position() {
                    Ok(p) => p,
                    Err(e) => {
                        log::error!("get localfile size faild : {}", e);
                        break;
                    }
                };

                if pos >= total_size {
                    break;
                }

                // check cancel
                {
                    let transfer = G_TRANSFER.read().unwrap();
                    if !transfer.contains_key(&local_path) {
                        log::debug!("user cancel transfer task");
                        break;
                    }
                }

                // update status
                if cur_timestamp_secs() - tick_time >= 1 {
                    let mut transfer = G_TRANSFER.write().unwrap();

                    if transfer.contains_key(&local_path) {
                        let item = transfer.get_mut(&local_path).unwrap();
                        item.speed = item.remaind_size - (total_size - pos) as f64;
                        item.remaind_size = (total_size - pos) as f64;
                        item.remaind_time = item.remaind_size / item.speed;

                        tick_time = cur_timestamp_secs();
                    }
                }
            }

            let mut transfer = G_TRANSFER.write().unwrap();

            if transfer.contains_key(&local_path) {
                transfer.remove(&local_path);
            }

            log::info!("get file worker finished");
        })
        .unwrap();

    Ok(())
}

use std::thread;
use std::sync::{Arc, };

// pub fn update_file_upload(file_upload: &FileUpload) {
 
//     file_upload.set_file_byte_upload(false);
//     println!("Controller.rs: file_byte_upload = {}", file_upload.get_file_byte_upload());
// }
pub fn proxy_run_req(
    sender: &Sender<FTPPacket>,
    file_upload_clone: Arc<Mutex<bool>>,
) -> Result<()> {
    // Создаем TCP-сервер для прокси
    let server = TcpListener::bind("127.0.0.1:0")?;
    let port = server.local_addr()?.port();

    // Формируем запрос на открытие туннеля
    let req = TunnelRequest { port };

    // Подготавливаем данные для отправки
    let sdads = "dsadsadas";
    let t_vec = sdads.as_bytes();

    // Отправляем PUT-запрос через sender
    sender.send(FTPPacket {
        id: 5,
        data: t_vec.to_vec(),
    });

    // Запускаем поток для обработки прокси
    let sender = sender.clone();

    thread::spawn(move || {
        // Принимаем соединение от сервера
        let (mut stream, _) = match server.accept() {
            Ok(conn) => conn,
            Err(e) => {
                log::error!("Failed to accept connection: {}", e);
                return;
            }
        };

        log::info!("Proxy started on port {}", port);

        // Отправляем данные клиенту
        let response_data = "HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\nHello, world!";
        if let Err(e) = stream.write_all(response_data.as_bytes()) {
            log::error!("Failed to send data to client: {}", e);
            return;
        }

        log::info!("Data sent to client");

        // Уведомляем об успешном запуске прокси
        let mut proxy_is = file_upload_clone.lock().unwrap();
        *proxy_is = true;

        log::info!("Proxy is running");
    });

    Ok(())
}