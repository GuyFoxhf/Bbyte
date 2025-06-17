use BByte_util::{ConnectionInfo, DrakulaProtocol};
use std::fs::File;
use std::io::{self, Read};
use crate::{G_DNA, msgbox};
// use serde::{Deserialize, Serialize};


use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use winapi::um::libloaderapi::{FindResourceW, LoadResource, LockResource, SizeofResource,LoadLibraryExW,LOAD_LIBRARY_AS_DATAFILE};
// use winapi::um::winbase::{};
use winapi::um::winnt::{LPCWSTR, };

pub fn master_configure() -> ConnectionInfo {


    unsafe {
    let exe_path = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            msgbox::info(&"Ошибка при получении пути к исполнимому файлу: {}".to_string(), &" size > 1024".to_string());
            eprintln!("Ошибка при получении пути к исполнимому файлу: {}", e);
                return ConnectionInfo {
                protocol: DrakulaProtocol::UDP.to_u8(),
                address: String::from("127.0.0.1:8001"),
                remark: String::from("Default"),
            };
        }
    };
    let exe_path_str = exe_path.to_string_lossy().to_string();

    let vite_path:Vec<u16> = exe_path_str.encode_utf16().collect();


    if let Some(data) = read_resource("res.json", "RES") {
        println!("Resource data: {:?}", data);
        let dataStr = String::from_utf8_lossy(&data);
        let JsonData: ConnectionInfo = serde_json::from_slice(dataStr.as_bytes()).unwrap();

    
        msgbox::info(&"runned1".to_string(), &"runned1".to_string());
        let flag: [u8; 8] = [0xFF, 0xFE, 0xF1, 0xA1, 0xFF, 0xFE, 0xF1, 0xA1];
       let addres = JsonData.address;
        let remark = JsonData.remark;
        let protocol = JsonData.protocol;
    
         return ConnectionInfo {
            protocol: protocol,
            // protocol: DrakulaProtocol::UDP.to_u8(),
            address: String::from(String::to_string(&addres)),
            remark: String::from(remark),
        };
        msgbox::info(&"data from (read_resource) ".to_string(), &dataStr.to_string());
    } else {
         // msgbox::info(&"data from (read_resource) failed ".to_string(), &"data from (read_resource) failed ".to_string());
    }

    let h_module = LoadLibraryExW(
        vite_path.as_ptr(),
        std::ptr::null_mut(),
        LOAD_LIBRARY_AS_DATAFILE
    );
    
    if h_module.is_null() {
        println!(":{}","Failed to load EXE");
        msgbox::info(&"exit".to_string(), &"Failed to load EXE".to_string());
        log::error!("parse master connection info data too long");
        std::process::exit(0);
     
    }

      let res_type = to_wide("RES");
      let res_name = to_wide("res.json");

      let h_res = FindResourceW(
          h_module,
          res_name.as_ptr() as LPCWSTR,
          res_type.as_ptr() as LPCWSTR
      );
      
      if h_res.is_null() {
        println!(":{}","(2)is_null resource");
        msgbox::info(&"exit".to_string(), &"is_null resource (2)".to_string());
        log::error!("parse master connection info data too long");
        std::process::exit(0);
      }

      let h_global = LoadResource(h_module, h_res);
      if h_global.is_null() {
        println!(":{}","(fiend) is_null resource");
        msgbox::info(&"exit".to_string(), &"is_null resource (fiend)".to_string());
        log::error!("parse master connection info data too long");
        std::process::exit(0);
      }

      let data_ptr = LockResource(h_global) as *const u8;
      let data_size = SizeofResource(h_module, h_res) as usize;

      let mut buffer = Vec::with_capacity(data_size);
      std::ptr::copy_nonoverlapping(data_ptr, buffer.as_mut_ptr(), data_size);
      buffer.set_len(data_size);

      let json_str = String::from_utf8(buffer).unwrap();
      println!("Resource content: {}", json_str);





    let JsonData: ConnectionInfo = serde_json::from_slice(json_str.as_bytes()).unwrap();


    let addres = JsonData.address;
    let remark = JsonData.remark;
    let protocol = JsonData.protocol;


    return ConnectionInfo {
        protocol: protocol,
        // protocol: DrakulaProtocol::UDP.to_u8(),
        address: String::from(String::to_string(&addres)),
        remark: String::from(remark),
    };

   
    }
}


pub fn read_resource(resource_name: &str, resource_type: &str) -> Option<Vec<u8>> {
    unsafe {
        let resource_name_wide: Vec<u16> = OsStr::new(resource_name).encode_wide().chain(Some(0)).collect();
        let resource_type_wide: Vec<u16> = OsStr::new(resource_type).encode_wide().chain(Some(0)).collect();

        let h_module = std::ptr::null_mut(); 
        let h_res = FindResourceW(h_module, resource_name_wide.as_ptr(), resource_type_wide.as_ptr());

        if h_res.is_null() {
            let form = format!("res type{}, res Name{}",resource_type,resource_name);
            eprintln!("Resource not found: {}:{}", resource_type, resource_name);
            return None;
        }

        let h_global = LoadResource(h_module, h_res);
        if h_global.is_null() {
            let form = format!("res type{}, res Name{}",resource_type,resource_name);
            eprintln!("Failed to load resource: {}:{}", resource_type, resource_name);
            return None;
        }
        let data_ptr = LockResource(h_global);
        if data_ptr.is_null() {
            let form = format!("res type{}, res Name{}",resource_type,resource_name);
            eprintln!("Failed to lock resource: {}:{}", resource_type, resource_name);
            return None;
        }
        let size = SizeofResource(h_module, h_res);
        if size == 0 {
            let form = format!("res type{}, res Name{}",resource_type,resource_name);
            eprintln!("Resource size is zero: {}:{}", resource_type, resource_name);
            return None;
        }
        let data = std::slice::from_raw_parts(data_ptr as *const u8, size.try_into().unwrap());
        Some(data.to_vec())
    }
}
fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}