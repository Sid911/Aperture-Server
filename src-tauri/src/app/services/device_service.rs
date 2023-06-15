use tauri::{AppHandle, Manager};
use tokio::sync::mpsc::{Receiver, Sender};

use tokio::sync::mpsc;

use crate::server::db::device_table::Device;
use tracing::info;

pub struct DeviceService {
    async_proc_input_rx: Receiver<Device>,
    async_proc_output_tx: Sender<Device>,
    async_proc_output_rx: Receiver<Device>,
}

impl DeviceService {
    pub fn new() -> (Sender<Device>, DeviceService) {
        let (inp_tx, inp_rx) = mpsc::channel::<Device>(1);
        let (out_tx, out_rx) = mpsc::channel::<Device>(1);
        (
            inp_tx,
            DeviceService {
                async_proc_input_rx: inp_rx,
                async_proc_output_rx: out_rx,
                async_proc_output_tx: out_tx,
            },
        )
    }

    async fn async_process_model(
        mut input_rx: mpsc::Receiver<Device>,
        output_tx: mpsc::Sender<Device>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        while let Some(input) = input_rx.recv().await {
            let output = input;
            output_tx.send(output).await?;
        }

        Ok(())
    }

    pub fn spawn_async_process(mut self, app_handle: AppHandle) {
        tauri::async_runtime::spawn(async move {
            Self::async_process_model(self.async_proc_input_rx, self.async_proc_output_tx).await
        });

        tauri::async_runtime::spawn(async move {
            loop {
                let Some(output) = (&mut self.async_proc_output_rx).recv().await else { continue };
                Self::rs2js(output, &app_handle);
            }
        });
    }

    fn rs2js<R: tauri::Runtime>(message: Device, manager: &impl Manager<R>) {
        info!(?message, "rs2js");
        manager.emit_all("rs2js", message).unwrap();
    }
}

#[tauri::command]
pub async fn js2rs(
    message: String,
    state: tauri::State<'_, DeviceInputTx<String>>,
) -> Result<(), String> {
    info!("{}:{}", message, "js2rs");
    let async_proc_input_tx = state.inner.lock().await;
    async_proc_input_tx
        .send(message)
        .await
        .map_err(|e| e.to_string())
}

use tauri::async_runtime::Mutex;

pub struct DeviceInputTx<T> {
    pub inner: Mutex<mpsc::Sender<T>>,
}
