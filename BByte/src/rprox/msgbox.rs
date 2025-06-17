use chrono::Local;
use std::io;
use std::path::Path;
use std::{fs::*, io::{Read, Write}};


pub fn info(title: &String, desc: &String) -> io::Result<()> {
    let log_file_path = Path::new("rust_log.log");
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)?;

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_entry = format!("[{}] [INFO] Title: {}, Description: {}\n", timestamp, title, desc);

   file.write_all(log_entry.as_bytes())?;
    Ok(())
}