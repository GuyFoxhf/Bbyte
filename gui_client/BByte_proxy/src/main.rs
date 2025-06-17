#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![windows_subsystem = "windows"]
use eframe::{egui, App};
use egui_extras::{Size, StripBuilder};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::net::{TcpStream, SocketAddr};

use std::time::Duration;
use BByte_util::{
    ftp::{
        method::{join_path, transfer_size, transfer_speed},
        FTPId, FTPPacket, FileInfo,
    },
    protocol::{tcp::TcpConnection, Client},
    rpc::{RpcClient, RpcMessage},
};
use BByte_util::packet::Message;
use crate::egui::RichText;
use crate::egui::Window;
use crate::egui::Color32;
use std::thread;
use std::sync::mpsc;
use rand::Rng; 
struct ReverseProxy {
    sender: String,
    socks5_status: Arc<Mutex<Option<bool>>>,
    socks5_address: String,
}

impl ReverseProxy {
    fn new(sender:String,ipPort:String) -> Self {
        Self {
            sender,
            socks5_status: Arc::new(Mutex::new(None)),
            socks5_address: ipPort,  
        }
    }

    fn check_socks5_status(&self, ctx: egui::Context) {
        let socks5_address = self.socks5_address.clone();
        let socks5_status = Arc::clone(&self.socks5_status);

        // Запускаем проверку в отдельном потоке
        thread::spawn(move || {
            let status = if let Ok(socket_addr) = socks5_address.parse::<SocketAddr>() {
                if let Ok(_stream) = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(2)) {
                    true
                } else {
                    false
                }
            } else {
                false
            };

            // Обновляем статус в основном потоке
            *socks5_status.lock().unwrap() = Some(status);

            // Запрашиваем перерисовку интерфейса
            ctx.request_repaint();
        });
    }

    // fn update_status(&self) {
    //     let status = self.check_socks5_status();
    //     *self.socks5_status.lock().unwrap() = Some(status);
    // }
}

impl eframe::App for ReverseProxy {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        Window::new("SOCKS5 Status")
        .resizable(false)  
        .title_bar(true) 
        .fixed_size(egui::Vec2::new(520.0, 90.0)).show(ctx, |ui| {
            if ui.button("Check Status").clicked() {
                self.check_socks5_status(ctx.clone());
            }

            let status = self.socks5_status.lock().unwrap();
            match *status {
                Some(true) => {
                    ui.label(RichText::new("Status: Working").color(Color32::GREEN));
                }
                Some(false) => {
                    ui.label(RichText::new("Status: Not Working").color(Color32::RED));
                }
                None => {
                    ui.label("Status: Unknown");
                }
            }

            ui.horizontal(|ui| {
                ui.label(format!("Address: {}", self.sender));
                if ui.button("Copy").clicked() {
                    if let Err(e) = arboard::Clipboard::new().and_then(|mut c| c.set_text(self.sender.clone())) {
                        log::error!("Failed to copy to clipboard: {}", e);
                    }
                }
            });
           
        });
    }
}
use local_ip_address::local_ip;
fn main() {
    #[cfg(debug_assertions)]
    {
        simple_logger::SimpleLogger::new()
            .with_threads(true)
            .with_utc_timestamps()
            .with_colors(true)
            .init()
            .unwrap();
        ::log::set_max_level(log::LevelFilter::Debug);
    }

    let args: Vec<String> = std::env::args().collect();


     
    let ss = format!("{}",args[0]);
    println!("{}",ss);

    let ss = format!("{}",args[1]);
    println!("{}",args.len());

    if args.len() < 2 {
        return;
    }

    // let mut s = TcpConnection::connect(&format!("127.0.0.1:{}", args[1])).unwrap();
    // let mut s2 = s.clone();

    // let title = args[2].clone();
    // std::thread::Builder::new()
    //     .name("ftp sender worker".to_string())
    //     .spawn(move || loop {
    //         let msg = match receiver.recv() {
    //             Ok(p) => p,
    //             Err(_) => {
    //                 std::process::exit(0);
    //             }
    //         };
    //         log::debug!("ftp send msg to core : {}", msg.id);
    //         s2.send(&mut msg.serialize().unwrap()).unwrap();
    //     })
    //     .unwrap();
    // let (sender, receiver) = mpsc::channel();
    
    // let mut rng = rand::thread_rng();
    // let random_id: u32 = rng.gen(); 
    // let random_data: Vec<u8> = (0..10).map(|_| rng.gen()).collect(); 


    // Создаем случайное сообщение
    // let random_message = Message {
    //     id: random_id,
    //     data: random_data,
    // };
    // sender.send(random_message.clone()).unwrap();
    // let my_local_ip = local_ip();
    let title = "Bbyte Reverse Proxy Beta";
    let my_local_ip = local_ip().unwrap().to_string();
        
    let proxy = format!("SOCKS5://{}:{}",my_local_ip,args[1]);
    let proxy2 = format!("127.0.0.1:{}", args[1]);

    let mut options = eframe::NativeOptions::default();
    options.initial_window_size = Some(egui::Vec2::new(285.0, 100.0)); // Размеры окна
    options.transparent = true;
    options.decorated = true; 
    options.resizable = false; 
    let title2 = "Bbyte Reverse Proxy Beta";
    // Запуск приложения
    eframe::run_native(
        title,
        options,
        Box::new(|_cc| Box::new(ReverseProxy::new(proxy,proxy2))),
    );
}