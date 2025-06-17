// async_module.rs
use tokio::runtime::Runtime;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use lazy_static::lazy_static;
use crate::rprox::main2;
lazy_static! {
    static ref RUNTIME: Runtime = Runtime::new().unwrap();
}

pub async fn start_async_task(args: Vec<String>, stop_rx: Arc<Mutex<mpsc::Receiver<()>>>) {

    let stop_rx_clone = stop_rx.clone();
    RUNTIME.spawn(async move {
          main2::rproxy(args, stop_rx_clone).await  
          
    });
}

// pub async fn start_async_task(args: Vec<String>, stop_rx: Arc<Mutex<mpsc::Receiver<()>>>) {
//     let stop_rx_clone = stop_rx.clone();
//     RUNTIME.spawn(async move {
//         main2::rproxy(args, stop_rx_clone).await
//     });
// }