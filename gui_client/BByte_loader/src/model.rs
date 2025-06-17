// model.rs
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct FileUpload {
    file_byte_upload: Arc<Mutex<bool>>,
}

impl FileUpload {
    pub fn new(file_byte_upload: bool) -> Self {
        FileUpload {
            file_byte_upload: Arc::new(Mutex::new(file_byte_upload)),
        }
    }

    pub fn set_file_byte_upload(&self, value: bool) {
        let mut data = self.file_byte_upload.lock().unwrap();
        *data = value;
    }

    pub fn get_file_byte_upload(&self) -> bool {
        *self.file_byte_upload.lock().unwrap()
    }
}