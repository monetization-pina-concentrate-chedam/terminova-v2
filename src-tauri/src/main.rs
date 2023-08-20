// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use {
    anyhow::Result,
    futures::io::{AsyncBufReadExt, BufReader},
    futures::{AsyncReadExt, AsyncWriteExt},
    interprocess::local_socket::tokio::LocalSocketStream,
    std::{io, sync::Arc},
    tauri::Manager,
    tokio::try_join,
};
static SERVER_LINE: &[u8] = b"Hello from server!\n";
static SERVER_BYTES: &[u8] = b"Bytes from server!\0";
static CLIENT_LINE: &[u8] = b"Hello from client!\n";
static CLIENT_BYTES: &[u8] = b"Bytes from client!\0";

#[tauri::command]
fn handle_message() {
    println!("I was invoked from JS!");
}

#[derive(Clone, serde::Serialize)]
struct Payload {
    message: String,
}

async fn client(name: Arc<String>) -> Result<(), io::Error> {
    let mut buffer = Vec::with_capacity(128);

    let (reader, mut writer) = LocalSocketStream::connect(name.as_str()).await?.split();
    let mut reader = BufReader::new(reader);

    let read = async {
        reader.read_until(b'\n', &mut buffer).await?;
        assert_eq!(buffer, SERVER_LINE);
        buffer.clear();

        reader.read_until(b'\0', &mut buffer).await?;
        assert_eq!(buffer, SERVER_BYTES);
        Result::<_, io::Error>::Ok(())
    };
    let write = async {
        writer.write_all(CLIENT_LINE).await?;

        writer.write_all(CLIENT_BYTES).await?;
        Ok(())
    };
    try_join!(read, write)?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let ctx = tauri::generate_context!();
    let app = tauri::Builder::default();
    app.invoke_handler(tauri::generate_handler![handle_message])
        .setup(|app| {
            let id = app.listen_global("eventname", |event| {
                let payload = event.payload().unwrap();
            });
            app.unlisten(id);
            app.emit_all(
                "eventname",
                Payload {
                    message: "Message".into(),
                },
            )
            .unwrap();
            Ok(())
        })
        .run(ctx)
        .expect("error while running tauri application");
}
