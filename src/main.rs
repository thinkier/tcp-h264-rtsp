extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tokio;
extern crate toml;

use std::error::Error;
use std::net::{IpAddr, SocketAddr};
use std::process::Stdio;
use std::sync::Arc;

use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpSocket;
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

use crate::model::config::Config;

mod model;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (terminator, mut terminator_rx) = mpsc::channel::<String>(1);

    let config = Arc::new(toml::from_slice::<Config>(
        &fs::read("./config.toml")
            .await
            .expect("failed to read config")
    ).expect("failed to parse config"));

    let mut joins = vec![];

    for (name, stream) in &config.stream {
        let (host, port) = stream.addr.split_at(stream.addr.find(':')
            .unwrap_or(stream.addr.len()));
        let host = host
            .parse()
            .expect("host is not an ip address");
        let port = port[1..].parse().unwrap_or(1264);

        match start_stream(Arc::clone(&config), terminator.clone(), name.to_owned(), host, port).await {
            Ok(join) => joins.push(join),
            Err(e) => eprintln!("Failed to connect to {}: {:?}", name, e)
        }
    }

    if joins.is_empty() {
        eprintln!("No sockets initialized, restarting server...");
    } else if let Some(crashed) = terminator_rx.recv().await {
        eprintln!("Caught termination of socket {}, restarting server...", crashed);
    } else {
        eprintln!("All sockets terminated, restarting server...");
    }

    for join in joins {
        join.abort();
    }

    Ok(())
}

async fn start_stream(config: Arc<Config>, terminator: Sender<String>, name: String, host: IpAddr, port: u16) -> Result<JoinHandle<()>, Box<dyn Error>> {
    let addr = SocketAddr::new(host, port);

    let mut stream = TcpSocket::new_v4()?
        .connect(addr)
        .await?;
    println!("Connected to {}", name);

    Ok(tokio::spawn(async move {
        let mut rtsp_server = Command::new("cvlc")
            .args(&[
                "-vvv",
                "stream:///dev/stdin",
                "--sout"
            ])
            .arg(format!("#rtp{{sdp=rtsp://:{}/{}}}", config.server.port, name))
            .arg(":demux=h264")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .expect("failed to spawn cvlc for the rtsp server");

        let mut h264_consumer = rtsp_server.stdin.take()
            .expect("cvlc didn't have an stdin available");

        'stream: loop {
            // 1MiB buffer
            let mut buf = Vec::with_capacity(1 << 20);

            let read = stream.read_buf(&mut buf).await;
            match read {
                Ok(read) => {
                    if read == 0 {
                        continue 'stream;
                    }
                }
                Err(e) => {
                    eprintln!("Disconnected {}: {:?}", name, e);
                    terminator.send(name)
                        .await
                        .expect("failed to send termination signal");
                    break 'stream;
                }
            }

            h264_consumer.write_all(&buf)
                .await
                .expect("failed to write video to the rtsp server");
        }
    }))
}
