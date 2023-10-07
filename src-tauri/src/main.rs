// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_static_resources;

mod app;
mod server;

use app::services::ip_service;
use server::db::device_table::Device;
use tauri::Manager;
use tokio::sync::Mutex;

struct TestWriter;

impl std::io::Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let buf_len = buf.len();
        println!("{:?}", buf);
        Ok(buf_len)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}


fn main() {
    #[cfg(release)]
    {
        let file_appender = tracing_appender::rolling::hourly("logs/", "prefix.log");
        tracing_subscriber::fmt().with_writer(file_appender).init();
    }
    #[cfg(not(release))]
    tracing_subscriber::fmt().init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![ip_service::get_ipv4,])
        .setup(|app| {
            let window = app.get_window("main").unwrap();
            let _main_window = app.get_window("main").unwrap();
            let app_handle = app.handle();
            // mount the rocket instance
            server::run(window);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
