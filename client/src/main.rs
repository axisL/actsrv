use std::error::Error;
use tokio::net::TcpStream;
use tokio::io::{Interest, AsyncWriteExt};
use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite};
use std::{env, io};
use futures::{StreamExt, Stream, Sink};
use futures::future;
use std::net::SocketAddr;
use bytes::Bytes;
use futures_util::SinkExt;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let stream = TcpStream::connect("127.0.0.1:6123").await.expect("connect to server failed");
    let mode = env::args()
        .nth(1)
        .unwrap_or_else(|| "c".to_string());
    match mode.as_str() {
        "o" => {
            send_once(stream).await;
        }
        "c" => {
            send_continues(stream).await?;
        }
        "w" => {
            write().await?;
        }
        "rw" => {
            read_write().await?;
        }
        _ => {
            send_continues(stream).await?;
        }
    }
    println!("fn main finished!");
    Ok(())
}

async fn send_continues(mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut need_send = true;
    loop {
        let ready = socket.ready(Interest::READABLE | Interest::WRITABLE).await?;
        if ready.is_readable() {
            let mut data = vec![0; 1024];
            match socket.try_read(&mut data) {
                Ok(n) => {
                    need_send = true;
                    println!("read length: {:?} of data: {:?}", n, data);
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    println!("read block");
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }

        if ready.is_writable() && need_send {
            // need_send = false;
            let to_send = b"wtf---0";
            socket.write_all(to_send).await?;
            println!("sent data: {:?}", to_send);
        } else {
            continue;
        }
    }
}


async fn send_once(mut socket: TcpStream) {
    let result = socket.write(b"wtf").await;
    println!("send ok: {:?}", result.is_ok());
    result.unwrap();
}

async fn write() -> Result<(), Box<dyn Error>> {
    // Connect to a peer
    let stream = TcpStream::connect("127.0.0.1:6123").await.expect("connect to server failed");
    // stream.into_split()
    loop {
        // Wait for the socket to be writable
        stream.writable().await?;

        // Try to write data, this may still fail with `WouldBlock`
        // if the readiness event is a false positive.
        match stream.try_write(b"hello world") {
            Ok(_n) => {
                break;// break or this will send all the time
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    Ok(())
}

async fn read_write() -> Result<(), Box<dyn Error>> {
    let mut socket = TcpStream::connect("127.0.0.1:6123").await?;
    let (r, w) = socket.split();
    let stream = FramedRead::new(r, BytesCodec::new());
    let _sink = FramedWrite::new(w, BytesCodec::new());
    let _st = stream.filter_map(|i| match i {
        Ok(i) => future::ready(Some(i.freeze())),
        Err(_e) => {
            future::ready(None)
        }
    });
    // .map(Ok);
    // match future::join(sink.send_all(&mut stdin), stdout.send_all(&mut stream)).await {
    //     (Err(e), _) | (_, Err(e)) => Err(e.into()),
    //     _ => ()
    // }
    Ok(())
}

pub async fn connect(
    addr: &SocketAddr,
    mut stdin: impl Stream<Item=Result<Bytes, io::Error>> + Unpin,
    mut stdout: impl Sink<Bytes, Error=io::Error> + Unpin,
) -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect(addr).await?;
    let (r, w) = stream.split();
    let mut sink = FramedWrite::new(w, BytesCodec::new());
    // filter map Result<BytesMut, Error> stream into just a Bytes stream to match stdout Sink
    // on the event of an Error, log the error and end the stream
    let mut stream = FramedRead::new(r, BytesCodec::new())
        .filter_map(|i| match i {
            //BytesMut into Bytes
            Ok(i) => future::ready(Some(i.freeze())),
            Err(e) => {
                println!("failed to read from socket; error={}", e);
                future::ready(None)
            }
        })
        .map(Ok);

    match future::join(sink.send_all(&mut stdin), stdout.send_all(&mut stream)).await {
        (Err(e), _) | (_, Err(e)) => Err(e.into()),
        _ => Ok(()),
    }
}