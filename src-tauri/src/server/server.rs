
use {
    futures::io::{AsyncBufReadExt, BufReader},
    futures::AsyncWriteExt,
    interprocess::local_socket::tokio::{LocalSocketListener, LocalSocketStream},
    std::io,
    tokio::{task, try_join},
};

pub async fn server() {
    async fn handle_conn(conn: LocalSocketStream) -> Result<(), io::Error> {
        let (reader, mut writer) = conn.into_split();
        let mut buffer = Vec::with_capacity(128);
        let mut reader = BufReader::new(reader);

        let read = async {
            reader.read_until(b'\n', &mut buffer).await?;
            buffer.clear();

            reader.read_until(b'\0', &mut buffer).await?;
            buffer.clear();
            Result::<_, io::Error>::Ok(())
        };
        let write = async {
            writer.write_all(b"hello\n").await?;
            writer.write_all(b"supp\0").await?;
            Result::<_, io::Error>::Ok(())
        };
        try_join!(read, write)?;
        Ok(())
    }

    let listener = LocalSocketListener::bind("terminova-007").unwrap();

    let mut tasks = Vec::new();

    let conn = listener.accept().await.unwrap();
    tasks.push(task::spawn(handle_conn(conn)));
    for task in tasks {
        let _ = task.await.unwrap();
    }
    ()
}

// #[tokio::main]
// async fn main() {
//     println!("Hello, world!");
//     server().await;
// }
