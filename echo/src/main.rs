use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};


#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6123").await.unwrap();
    println!("Hello, world!");
    loop {
        let (socket,_) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) {
    let mut connection = Connection::new(socket);
    if let Some(frame)= connection.read_frame().await.unwrap(){
        println!("receive: {:?}",frame);
        let response:Frame = frame;
        connection.write_frame(&response).await.unwrap();
    }
}
