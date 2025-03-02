use std::io::ErrorKind;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn read_int(socket: &mut TcpStream, mut buf: &mut [u8]) -> Result<usize, std::io::Error> {
    let n = socket.read(&mut buf).await?;
    if 0 == n {
        return Err(std::io::Error::from(ErrorKind::ConnectionAborted));
    }
    let x: usize = str::from_utf8(&buf[0..n]).unwrap().trim().parse().unwrap();
    Ok(x)
}

async fn read_x_and_y_and_reply_with_sum(
    mut socket: &mut TcpStream,
    mut buf: &mut [u8],
) -> Result<(), std::io::Error> {
    socket.write_all("< x = ".as_bytes()).await?;
    let x = read_int(&mut socket, &mut buf).await?;
    socket.write_all("< y = ".as_bytes()).await?;
    let y = read_int(&mut socket, &mut buf).await?;

    let z = x + y;

    // Write the data back
    socket
        .write_all(format!("> z = {}\n", z).as_bytes())
        .await?;

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("listening on {}", listener.local_addr()?);

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 10];

            // In a loop, read data from the socket and write the data back.
            loop {
                if let Err(e) = read_x_and_y_and_reply_with_sum(&mut socket, &mut buf).await {
                    eprintln!("socket failure; err = {:?}", e);
                    return;
                }
            }
        });
    }
}
