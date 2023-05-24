// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[macro_use] extern crate rocket;

#[macro_use] extern crate rocket_include_static_resources;
mod server;


use rocket::{async_trait, route::Handler, Request, Route, http::Method};
use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}


#[derive(Clone)]
struct WindowHandler {
    window: tauri::Window,
}

impl WindowHandler {
    fn new(window: tauri::Window) -> Self {
        Self { window }
    }
}

#[async_trait]
impl Handler for WindowHandler {
    async fn handle<'r>(&self, request: &'r Request<'_>, data: rocket::Data<'r>) -> rocket::route::Outcome<'r>  {
        self.window
            .emit("from-rust", format!("message"))
            .expect("failed to emit");
        todo!()
    }
}
impl From<WindowHandler> for Vec<Route> {
    fn from(value: WindowHandler) -> Self {
        vec![Route::new(Method::Get, "/", value)]
    }
}


fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            let window = app.get_window("main").unwrap();

            let index = WindowHandler::new(window);
            // mount the rocket instance
            tauri::async_runtime::spawn(async move {
                let _rocket = server::rocket();
                _rocket.launch().await
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
