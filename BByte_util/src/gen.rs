use crate::{ConnectionInfo, SlaveDNA};
use std::io::*;


use std::fs::File;
use std::io::Read;
use winapi::um::winbase::{UpdateResourceW, BeginUpdateResourceW, EndUpdateResourceW};
use winapi::um::winnt::{LPCWSTR, PVOID};
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;
use std::ptr::{self, null};
use std::fs;
pub const CONNECTION_INFO_FLAG: [u8; 8] = [0xff, 0xfe, 0xf1, 0xa1, 0xff, 0xfe, 0xf1, 0xa1];
use std::{ thread, time::Duration};

const MAX_RETRIES: u32 = 10;
const RETRY_DELAY_MS: u64 = 900;

fn wait_for_file(path: &str) -> std::result::Result<(), std::io::Error> {
        for _ in 0..MAX_RETRIES {
        if let Ok(metadata) = fs::metadata(path) {
            if metadata.is_file() {
                return Ok(());
            }
        }
        thread::sleep(Duration::from_millis(RETRY_DELAY_MS));
    }
   
    Err(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        format!("File {} not found after {} retries", path, MAX_RETRIES),
    ))
}

pub fn replace_connection_info_to_new_file(
    path: &String,
    new_path: &String,
    new_info: ConnectionInfo,
) -> Result<()> {

    let mut f = File::open(path)?;

    // Читаем данные из исходного файла
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    // f.sync_all()?;
    // drop(f);
     
    {
        let mut new_f = File::create(new_path)?;
        new_f.write_all(&buf)?;
        // Явная синхронизация файловой системы
        new_f.sync_all()?;
        drop(new_f)
    } // new_f выходит из области видимости и файл закрывается

    // // Ожидаем появления файла
    // wait_for_file(new_path)?;
 

    // return ConnectionInfo::clone();



    let payload = new_info.serialize()?;
    let data = payload;
    // fs::write("./res/res.json", payload)?;
    // if let Ok(file) = File::open("./res/res.json") {
    //     file.sync_all()?;
    //     drop(file)
    // }
    // wait_for_file("./res/res.json")?;
    // // начало импорта ресурса
    // let mut file = File::open("./res/res.json")?;
    // let mut data = Vec::new();
    // file.read_to_end(&mut data)?;
    // drop(file);

    // let file_name = std::path::Path::new(&path).file_name().unwrap().to_str().unwrap();
    // 2. Подготавливаем имя файла и ресурса
    let exe_path = to_wide(new_path);
    let resource_type = to_wide("RES"); // Тип ресурса (можно использовать стандартные типы)
    let resource_name = to_wide("res.json");

    // 3. Начинаем обновление ресурсов
    unsafe {
        let handle = BeginUpdateResourceW(exe_path.as_ptr(), false as _);
        if handle.is_null() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "BeginUpdateResource failed",
            ));
             
        }

        // 4. Обновляем ресурс
        let success = UpdateResourceW(
            handle,
            resource_type.as_ptr() as LPCWSTR,
            resource_name.as_ptr() as LPCWSTR,
            0x0409, // Язык (английский США)
            data.as_ptr() as PVOID,
            data.len() as u32
        );

        if success == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "UpdateResourceW failed",
            ));
            
        }

        // 5. Завершаем обновление
        if EndUpdateResourceW(handle, false as _) == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "EndUpdateResource failed",
            ));
             
        }
    }
















    // let mut f = std::fs::File::open(path)?;

    // let mut buf = vec![];
    // let mut size = f.read_to_end(&mut buf)?;
    

    // let mut cursor = Cursor::new(&mut buf);

    // let mut found = false;

    // while size >= 8 {
    //     let mut flag = [0u8; 8];

    //     cursor.read_exact(&mut flag)?;

    //     if flag == CONNECTION_INFO_FLAG {
    //         cursor.seek(SeekFrom::Current(-8))?;

    //         let payload = new_info.serialize()?;

    //         cursor.write_all(&SlaveDNA::new(&payload).serilize())?;

    //         found = true;

    //         break;
    //     }

    //     cursor.seek(SeekFrom::Current(-7))?;

    //     size -= 1;
    // }

    // if !found {
    //     return Err(std::io::Error::new(
    //         std::io::ErrorKind::NotFound,
    //         "Not found flag",
    //     ));
    // }

    // let mut new_f = std::fs::File::create(new_path)?;

    // new_f.write_all(&mut buf)?;

    Ok(())
}
fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0).into_iter()).collect()
}