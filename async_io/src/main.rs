use std::io::ErrorKind;
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt, Stdin};
use tokio::net::{TcpListener, TcpStream};

#[derive(Default)]
struct LeSharedState {
    counter: usize,
    x: usize,
    y: usize,
}

fn exchange(current: &mut usize, new: &usize) -> usize {
    let old_current = *current;
    *current = *new;
    old_current
}

impl LeSharedState {
    fn inc_counter(&mut self) -> usize {
        self.counter += 1;
        self.counter
    }

    fn set_x(&mut self, x: usize) -> usize {
        exchange(&mut self.x, &x)
    }

    fn set_y(&mut self, y: usize) -> usize {
        exchange(&mut self.y, &y)
    }

    fn get_z(&self) -> usize {
        self.x + self.y
    }
}

type State = Arc<Mutex<LeSharedState>>;

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
    task_state: &mut State,
) -> Result<(), std::io::Error> {
    {
        socket.write_all("< x = ".as_bytes()).await?;
        let x = read_int(&mut socket, &mut buf).await?;
        l(task_state).set_x(x);
    }
    {
        socket.write_all("< y = ".as_bytes()).await?;
        let y = read_int(&mut socket, &mut buf).await?;
        l(task_state).set_y(y);
    }

    // Write the data back
    let z = l(task_state).get_z();
    socket
        .write_all(format!("> z = {}\n", z).as_bytes())
        .await?;

    Ok(())
}

fn l(state: &mut State) -> std::sync::MutexGuard<'_, LeSharedState> {
    state.lock().unwrap()
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("listening on {}", listener.local_addr()?);

    // TODO send event from main thread

    let mut le_state = State::default();

    loop {
        let (mut socket, _) = listener.accept().await?;
        println!("new connection {}", l(&mut le_state).inc_counter());

        let mut task_state = le_state.clone();

        tokio::spawn(async move {
            let mut buf = [0; 10];

            // In a loop, read data from the socket and write the data back.
            loop {
                if let Err(e) =
                    read_x_and_y_and_reply_with_sum(&mut socket, &mut buf, &mut task_state).await
                {
                    eprintln!("socket failure; err = {:?}", e);
                    return;
                }
            }
        });
    }
}
