#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::{
    collections::HashMap,
    sync::{
        mpsc::{channel, Sender},
        Arc, RwLock,
    },
};

use eframe::{egui, App};
use egui_extras::{Size, StripBuilder};
use BByte_util::{
    ftp::{
        method::{join_path, transfer_size, transfer_speed},
        FTPId, FTPPacket, FileInfo,
    },
    protocol::{tcp::TcpConnection, Client},
    rpc::{RpcClient, RpcMessage},
};
use lazy_static::*;

mod controller;
mod msgbox;
use controller::*;
mod lib;
mod model;

use model::FileUpload;

lazy_static! {
    static ref G_RPCCLIENT: Arc<RpcClient> = Arc::new(RpcClient::new());
    static ref G_TRANSFER: Arc<RwLock<HashMap<String, TransferInfo>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
enum SwitchDock {
    List,
    Transfer,
}

#[derive(PartialEq)]
enum FSType {
    Local,
    Remote,
}

impl std::fmt::Debug for FSType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "Local FS"),
            Self::Remote => write!(f, "Remote FS"),
            
        }
    }
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
struct FtpApp {
    selected_program: String,
    file_byte_upload: Arc<Mutex<bool>>,
    selected_file: String,
    initilized: bool,
    switch: SwitchDock,
    title: String,
    local_path: String,
    remote_path: String,
    // local_disk_info: Vec<FileInfo>,
    // remote_disk_info: Vec<FileInfo>,
    sender: Sender<FTPPacket>,
    drive_image: egui_extras::RetainedImage,
    folder_image: egui_extras::RetainedImage,
    file_image: egui_extras::RetainedImage,
    local_folder_strace: Vec<String>,
    remote_folder_strace: Vec<String>,
}

impl FtpApp {
    const ROOT_FLAG: &'static str = "[DISK]";
 
    pub fn new(sender: Sender<FTPPacket>) -> Self {
        let remote_disk_info = match get_remote_disk_info(&sender) {
            Ok(p) => p,
            Err(e) => {
                msgbox::error(
                    &"Bbyte FTP".to_string(),
                    &format!("get disk info error : {}", e),
                );
                std::process::exit(0);
            }
        };

        // let local_disk_info = get_local_disk_info().unwrap();

        Self {
            initilized: false,
            selected_program: String::new(),
            selected_file: String::new(),
            file_byte_upload: Arc::new(Mutex::new(false)),
            switch: SwitchDock::List,
            local_path: String::from(FtpApp::ROOT_FLAG),
            remote_path: String::from(FtpApp::ROOT_FLAG),
            sender,
            drive_image: egui_extras::RetainedImage::from_image_bytes(
                "drive.ico",
                include_bytes!("res/drive.ico"),
            )
            .unwrap(),
            folder_image: egui_extras::RetainedImage::from_image_bytes(
                "folder.ico",
                include_bytes!("res/folder.ico"),
            )
            .unwrap(),
            file_image: egui_extras::RetainedImage::from_image_bytes(
                "file.ico",
                include_bytes!("res/file.ico"),
            )
            .unwrap(),
            title: String::from("Bbyte FTP"),
            local_folder_strace: vec![],
            remote_folder_strace: vec![],
        }
 
    }
}

impl App for FtpApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.initilized {
                ui.ctx().set_visuals(egui::Visuals::dark());
                self.initilized = true;
            }

            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.switch, SwitchDock::Transfer, "Loader");
                // ui.selectable_value(&mut self.switch, SwitchDock::List, "List");

                // self.switch = SwitchDock::List;

                let visuals = ui.ctx().style().visuals.clone();
                match visuals.light_dark_small_toggle_button(ui) {
                    Some(v) => ui.ctx().set_visuals(v),
                    None => {}
                };
            });

            ui.separator();
 
           
            match self.switch {
                SwitchDock::Transfer => {
                    StripBuilder::new(ui)
                        .size(Size::exact(30.0))
                        .size(Size::exact(10.0))
                        .size(Size::remainder())
                        .vertical(|mut strip| {
                            strip.cell(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.heading("Loader List");
                                });
                            });
                            strip.cell(|ui| {
                                ui.separator();
                            });
                            strip.cell(|ui| {
    
                                self.render_combobox_with_label(ui, 200.0, 125.0);
                                self.transfer_table("3", ctx, ui);
                            });
                        });
                }
                SwitchDock::List => {
 
                }
            }
        });

        ctx.request_repaint();
        self.switch = SwitchDock::Transfer;
    }
}

impl FtpApp {
    fn render_combobox_with_label(&mut self, ui: &mut egui::Ui, x: f32, y: f32) {
         ui.vertical(|ui| {
            ui.separator();   
        });
        ui.horizontal(|ui| {
 
            let button_pos = egui::Pos2::new(x -160.0, y-30.0);
            ui.allocate_ui_at_rect(
                egui::Rect::from_min_size(button_pos, egui::vec2(100.0, 30.0)),
                |ui| {
                    if ui.button("Select File").clicked() {
                         let path = std::env::current_dir().unwrap();
                        if let Some(file_path) = rfd::FileDialog::new()
                            .set_directory(&path)
                            .pick_file()
                        {
                             println!("Selected File: {:?}", file_path);
                            self.selected_file = file_path.display().to_string();
                         }
                    }
                },
            );
            let file_label_pos = egui::Pos2::new(x - 87.0, y - 25.0);
            ui.painter().text(
                file_label_pos,
                egui::Align2::LEFT_TOP,
                format!("Payload Path: {}", self.selected_file),
                egui::TextStyle::Heading.resolve(&ui.style()), 
                ui.style().visuals.text_color(),
            );
  
        });

        ui.vertical(|ui| {
            
            ui.separator();  
           
        });
        let available_width = ui.available_width(); 
        let button_width = 150.0;
        let button_height = 60.0; 
        
        // Центрируем по X и Y
        let button_x = (available_width - 20.0) / 2.0; 
        let button_y = 227.00; 
        let file_byte_upload = self.file_byte_upload.lock().unwrap();
        // println!("main: file_byte_upload = {}", *file_byte_upload);
        ui.allocate_ui_at_rect(
            egui::Rect::from_min_size(egui::Pos2::new(button_x, button_y), egui::vec2(button_width, button_height)),
            |ui| {
                let is_inject_enabled = !self.selected_file.is_empty() ;//&& !self.selected_program.is_empty();
                let button = egui::Button::new(
                    egui::RichText::new("Load")
                        .size(19.0) 
                );
        
        
                let button = if is_inject_enabled {
                    button.fill(egui::Color32::RED)
                } else {
                    button.fill(egui::Color32::DARK_BLUE)
                };
        
            
                if ui.add_enabled(is_inject_enabled, button).clicked() {
                    // Логика для инъекции
                    println!(
                        "Load file: {} into process: {}",
                        self.selected_file, self.selected_program
                    );
                    self.remote_path ="C:\\".to_string();
                    if self.local_path != "1"//FtpApp::ROOT_FLAG
                        && self.remote_path != "1"//FtpApp::ROOT_FLAG
                    {
                        
                        // let file_upload = FileUpload::new(false);

                        // file_upload.set_file_byte_upload(false);
                     
                        // Передаем структуру в controller.rs
                        // controller::update_file_upload(&file_upload);
                        let file_upload_clone = Arc::clone(&self.file_byte_upload);
  
                        match upload_file(
                            &self.sender,
                            &self.selected_file,
                            &self.selected_program,
                            file_upload_clone
                        ) {
                            Ok(_) => {
                                self.switch = SwitchDock::Transfer;
                                print!("OK Upload")
                                // self.file_byte_upload = true;
                                // msgbox::_info(&"download".to_string(), &"download ok".to_string())
                            }
                            Err(e) => {
                                msgbox::error(
                                    &self.title.to_string(),
                                    &format!(
                                        "download remote file faild : {}",
                                        e
                                    ),
                                );
                                ui.close_menu();
                                return;
                            }
                        };
                    } else {
                        msgbox::error(
                            &self.title,
                            &"not allow download to root path".to_string(),
                        );
                    }

                    ui.close_menu();
                 
                }
            },
        );
        
        ui.label(format!("file_byte_upload: {}", *file_byte_upload));


    }
    fn render_file_table(&mut self, id: &str, ctx: &egui::Context, ui: &mut egui::Ui, typ: FSType) {
        egui::CentralPanel::default()
            .show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(format!("{:?}", typ));
                });
                ui.separator();

                StripBuilder::new(ui)
                    .size(Size::exact(20.0))
                    .size(Size::exact(5.0))
                    .size(Size::remainder())
                    .vertical(|mut strip| {
 
                        
                        
                        strip.strip(|builder| {
                            builder
                                .size(Size::exact(80.0))
                                .size(Size::remainder())
                                .size(Size::exact(120.0))
                                .horizontal(|mut strip| {
                                    strip.cell(|ui| {
                                        ui.label("Current Path:");
                                    });
                                    
                                    strip.cell(|ui| {
                                        if typ == FSType::Local {
                                            ui.label(&self.local_path);
                                        } else {
                                            ui.label(&self.remote_path);
                                        }
                                    });

       
                                strip.strip(|builder|{
                                    builder
                                    .size(Size::exact(60.0))
                                    .size(Size::exact(60.0))
                                    .horizontal(|mut strip|{
                                        strip.cell(|ui|{
                                            if ui.button("Refresh").clicked(){
                                                if typ == FSType::Local{
                                                    // self.local_disk_info = FtpApp::refresh_local_path(&self.local_path);
                                                } else {
                                                    // self.remote_disk_info = FtpApp::refresh_remote_path(&self.remote_path , &self.sender);
                                                }
                                            }
                                        });


            
                                        
                                    });



                                });     
                    });
                });
                strip.cell(|ui|{
                    ui.separator();
                });
                strip.strip(|builder| {
                    builder
                    .size(Size::remainder())
                    .vertical(|mut strip|{
                        strip.cell(|ui|{
                            if typ == FSType::Remote{
                                self.file_table(id,ctx, ui, typ);
                            } else {
                                self.file_table(id,ctx, ui , typ);
                            }
                        });
                    });
                });
            });
        });
           
    }

    fn file_table(&mut self, id: &str, ctx: &egui::Context, ui: &mut egui::Ui, typ: FSType) {
        ui.push_id(id, |ui| {
            egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Size::initial(50.0).at_least(50.0))
                .column(Size::initial(110.0).at_least(50.0))
                .column(Size::initial(50.0).at_least(50.0))
                .column(Size::initial(90.0).at_least(50.0))
                .column(Size::initial(165.0).at_least(50.0))
                .resizable(true)
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("");
                    });
                    header.col(|ui| {
                        ui.heading("Name");
                    });
                    header.col(|ui| {
                        ui.heading("Type");
                    });
                    header.col(|ui| {
                        ui.heading("Size");
                    });
                    header.col(|ui| {
                        ui.heading("Last Modified");
                    });
                })
                .body(|mut body| {
                    let files = if typ == FSType::Remote {
                       
                    } else {
                        
                    };

                   
                });
        });
    }

    fn transfer_table(&mut self, id: &str, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.push_id(id, |ui| {
            egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Size::initial(50.0).at_least(50.0))
                .column(Size::initial(220.0).at_least(50.0))
                .column(Size::initial(220.0).at_least(50.0))
                .column(Size::initial(80.0).at_least(50.0))
                .column(Size::initial(100.0).at_least(50.0))
                .column(Size::initial(100.0).at_least(50.0))
                .column(Size::initial(150.0).at_least(50.0))
                .column(Size::initial(55.0).at_least(50.0))
                .resizable(true)
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("");
                    });
                    header.col(|ui| {
                        ui.heading("Remote Path");
                    });
                    header.col(|ui| {
                        ui.heading("Local Path");
                    });
                    header.col(|ui| {
                        ui.heading("Type");
                    });
                    header.col(|ui| {
                        ui.heading("Size");
                    });
                    header.col(|ui| {
                        ui.heading("Speed");
                    });
                    header.col(|ui| {
                        ui.heading("Remaind Time");
                    });
                    header.col(|ui| {
                        ui.heading("");
                    });
                })
                .body(|mut body| {
                    let transfer_lock = G_TRANSFER.read().unwrap();
                    let transfer = HashMap::clone(&transfer_lock);
                    drop(transfer_lock);
           
                    for (_, i) in transfer {
                        let row_height = 20.0;
                        body.row(row_height, |mut row| {
                            row.col(|ui| {
                                ui.add(egui::Image::new(
                                    self.file_image.texture_id(ctx),
                                    egui::Vec2::new(20.0, 20.0),
                                ));
                            });
                            row.col(|ui| {
                                ui.label(&i.remote_path);
                            });
                            row.col(|ui| {
                                ui.label(&i.local_path);
                            });
                            row.col(|ui| {
                                ui.label(&i.typ);
                            });
                            row.col(|ui| {
                                ui.label(transfer_size(i.size));
                            });

                            row.col(|ui| {
                                ui.label(transfer_speed(i.speed));
                            });

                            row.col(|ui| {
                                ui.label(format!("{} s", i.remaind_time as i64));
                            });

                            row.col(|ui| {
                                if ui.button("Cancel").clicked() {
                                    let mut transfer = G_TRANSFER.write().unwrap();
                                    if transfer.contains_key(&i.local_path) {
                                        transfer.remove(&i.local_path);
                                    }
                                };
                            });
                        });
                    }
                
                });
        });
    }
    fn refresh_local_path(local_path: &String) -> Vec<FileInfo> {
        if local_path == FtpApp::ROOT_FLAG {
            
        }

        match get_local_folder_info(local_path) {
            Ok(p) => p,
            Err(e) => {
                msgbox::error(
                    &"Bbyte FTP".to_string(),
                    &format!("get folder info faild : {}", e),
                );
                vec![]
            }
        }
    }
    fn refresh_remote_path(remote_path: &String, sender: &Sender<FTPPacket>) -> Vec<FileInfo> {
        if remote_path == FtpApp::ROOT_FLAG {
            match get_remote_disk_info(sender) {
                Ok(p) => p,
                Err(e) => {
                    msgbox::error(
                        &"Bbyte FTP".to_string(),
                        &format!("get disk info error : {}", e),
                    );
                    vec![]
                }
            };
        }

        match get_remote_folder_info(sender, remote_path) {
            Ok(p) => p,
            Err(e) => {
                msgbox::error(
                    &"Bbyte FTP".to_string(),
                    &format!("get remote folder info faild : {}", e),
                );
                vec![]
            }
        }
    }
}

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

    if args.len() < 3 {
        return;
    }
  
 

    let mut s = TcpConnection::connect(&format!("127.0.0.1:{}", args[1])).unwrap();
    let mut s2 = s.clone();

    let title = args[2].clone();

    std::thread::Builder::new()
        .name("ftp receiver worker".to_string())
        .spawn(move || loop {
            let data = match s.recv() {
                Ok(p) => p,
                Err(_) => {
                    std::process::exit(0);
                }
            };
            let packet = FTPPacket::parse(&data).unwrap();

            match packet.id() {
                FTPId::RPC => {
                    let msg = RpcMessage::parse(&packet.data).unwrap();
                    log::debug!("ftp recv msg from core : {}", msg.id);
                    G_RPCCLIENT.write(&msg);
                }
                FTPId::Close => {
                    std::process::exit(0);
                }
                FTPId::Get => {}
                FTPId::Put => {}
                FTPId::Unknown => {}
            }
        })
        .unwrap();

    let (sender, receiver) = channel::<FTPPacket>();

    std::thread::Builder::new()
        .name("ftp sender worker".to_string())
        .spawn(move || loop {
            let msg = match receiver.recv() {
                Ok(p) => p,
                Err(_) => {
                    std::process::exit(0);
                }
            };
            log::debug!("ftp send msg to core : {}", msg.id);
            s2.send(&mut msg.serialize().unwrap()).unwrap();
        })
        .unwrap();
    let file_upload = Arc::new(FileUpload::new(false));
    // println!("Main.rs: file_byte_upload = {}", file_upload.get_file_byte_upload());

    let mut options = eframe::NativeOptions::default();
    options.initial_window_size = Some(egui::Vec2::new(1070.0, 500.0));
    eframe::run_native(
        &format!("Bbyte Loader 1.2 - {}", title),
        options,
        Box::new(|_cc| Box::new(FtpApp::new(sender))),
    );
}
