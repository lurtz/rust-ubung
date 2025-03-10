use std::io::{stdin, stdout, ErrorKind, Write};
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch as Channel_type;

struct LeSharedState {
    counter: usize,
    x: usize,
    y: usize,
    sender: Channel_type::Sender<String>,
}

impl Default for LeSharedState {
    fn default() -> Self {
        let sender = Channel_type::Sender::<String>::new("".to_string());
        Self {
            counter: Default::default(),
            x: Default::default(),
            y: Default::default(),
            sender,
        }
    }
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

    fn send_event(&self, event: &str) -> Result<(), Channel_type::error::SendError<String>> {
        self.sender.send(event.to_string())
    }

    fn get_event_update_receiver(&self) -> Channel_type::Receiver<String> {
        self.sender.subscribe()
    }
}

type State = Arc<Mutex<LeSharedState>>;

async fn read_int(socket: &mut TcpStream, mut buf: &mut [u8]) -> Result<usize, std::io::Error> {
    let n = socket.read(&mut buf).await?;
    if 0 == n {
        return Err(std::io::Error::from(ErrorKind::ConnectionAborted));
    }
    let x: usize = std::str::from_utf8(&buf[0..n])
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    Ok(x)
}

async fn read_int_and_watch_for_event(
    mut socket: &mut TcpStream,
    mut buf: &mut [u8],
    event_receiver: &mut Channel_type::Receiver<String>,
) -> Result<usize, std::io::Error> {
    enum SelectKind {
        ReadInt(usize),
        Event(String),
    }

    let n;
    loop {
        let select_kind = tokio::select! {
            x = read_int(&mut socket, &mut buf) => SelectKind::ReadInt(x?),
            _ = event_receiver.changed() => {SelectKind::Event(event_receiver.borrow_and_update().clone())}
        };
        match select_kind {
            SelectKind::ReadInt(xx) => {
                n = xx;
                break;
            }
            SelectKind::Event(s) => {
                socket
                    .write_all(format!("\n got event: {}\n", s).as_bytes())
                    .await?;
            }
        }
    }
    Ok(n)
}

async fn read_x_and_y_and_reply_with_sum(
    socket: &mut TcpStream,
    buf: &mut [u8],
    task_state: &mut State,
    event_receiver: &mut Channel_type::Receiver<String>,
) -> Result<(), std::io::Error> {
    {
        socket.write_all("< x = ".as_bytes()).await?;
        let x = read_int_and_watch_for_event(socket, buf, event_receiver).await?;
        l(task_state).set_x(x);
    }
    {
        socket.write_all("< y = ".as_bytes()).await?;
        let y = read_int_and_watch_for_event(socket, buf, event_receiver).await?;
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

fn io_thread_main(mut thread_state: &mut State) {
    let mut buffer = String::new();
    buffer.reserve(10);
    loop {
        print!("Enter event content: ");
        stdout().flush().expect("flush failed");
        stdin()
            .read_line(&mut buffer)
            .expect("no proper string entered");

        l(&mut thread_state).send_event(buffer.trim()).unwrap();
        buffer.clear();
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("listening on {}", listener.local_addr()?);

    let mut le_state = State::default();

    // send event from user io thread
    let mut thread_state = le_state.clone();
    let _le_thread = std::thread::spawn(move || {
        io_thread_main(&mut thread_state);
    });

    loop {
        let (mut socket, _) = listener.accept().await?;
        println!("new connection {}", l(&mut le_state).inc_counter());

        let mut task_state = le_state.clone();

        tokio::spawn(async move {
            let mut buf = [0; 10];
            let mut event_receiver = l(&mut task_state).get_event_update_receiver();

            // In a loop, read data from the socket and write the data back.
            loop {
                if let Err(e) = read_x_and_y_and_reply_with_sum(
                    &mut socket,
                    &mut buf,
                    &mut task_state,
                    &mut event_receiver,
                )
                .await
                {
                    eprintln!("socket failure; err = {:?}", e);
                    return;
                }
            }
        });
    }
}
