use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::env;

#[tokio::main]
async fn main() {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6123".to_string());
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("listen on {:?}", addr);
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(mut socket: TcpStream) {
    let mut buffer = Vec::new();
    socket.read_to_end(&mut buffer).await.unwrap();
    println!("receive {:?}",buffer);
    socket.write(&buffer).await.unwrap();
    println!("write {:?}",buffer);
}
