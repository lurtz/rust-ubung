use std::io::{stdin, stdout, ErrorKind, Write};
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::signal;
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

async fn read_int<Reader>(socket: &mut Reader, buf: &mut [u8]) -> Result<usize, std::io::Error>
where
    Reader: AsyncReadExt + Unpin,
{
    let n = socket.read(buf).await?;
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

async fn read_int_and_watch_for_event<Socket>(
    socket: &mut Socket,
    buf: &mut [u8],
    event_receiver: &mut Channel_type::Receiver<String>,
) -> Result<usize, std::io::Error>
where
    Socket: AsyncReadExt + AsyncWriteExt + Unpin,
{
    let n;
    loop {
        tokio::select! {
            x = read_int(socket, buf) => {n=x?; break;},
            _ = event_receiver.changed() => {
                socket
                    .write_all(format!("\n got event: {}\n", *event_receiver.borrow_and_update()).as_bytes())
                    .await?;
            }
        };
    }
    Ok(n)
}

async fn read_x_and_y_and_reply_with_sum<Socket>(
    socket: &mut Socket,
    buf: &mut [u8],
    task_state: &mut State,
    event_receiver: &mut Channel_type::Receiver<String>,
) -> Result<(), std::io::Error>
where
    Socket: AsyncReadExt + AsyncWriteExt + Unpin,
{
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

fn l(state: &State) -> std::sync::MutexGuard<'_, LeSharedState> {
    state.lock().unwrap()
}

fn io_thread_main(thread_state: &mut State) {
    let mut buffer = String::new();
    buffer.reserve(10);
    loop {
        print!("Enter event content: ");
        stdout().flush().expect("flush failed");
        stdin()
            .read_line(&mut buffer)
            .expect("no proper string entered");

        let _ = l(thread_state).send_event(buffer.trim());
        buffer.clear();
    }
}

fn create_new_connection_handler<Socket>(le_state: State) -> impl Fn(Socket)
where
    Socket: AsyncReadExt + AsyncWriteExt + Unpin + Send + 'static,
{
    move |mut socket| {
        println!("new connection {}", l(&le_state).inc_counter());
        let mut task_state = le_state.clone();

        tokio::spawn(async move {
            let mut buf = vec![0; 10];
            let mut event_receiver = l(&task_state).get_event_update_receiver();

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
                    break;
                }
            }
        });
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("listening on {}", listener.local_addr()?);

    let le_state = State::default();

    // send event from user io thread, cannot be managed by tokio, because
    // reading from stdin blocks. Due to that the runtime will not shutdown
    // without user input. But with a normal os thread the application
    // terminates as expected.
    let mut thread_state = le_state.clone();
    std::thread::spawn(move || {
        io_thread_main(&mut thread_state);
    });

    let handle_new_connection = create_new_connection_handler(le_state);

    tokio::spawn(async move {
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            handle_new_connection(socket);
        }
    });

    signal::ctrl_c().await.unwrap();
    println!("terminating");

    Ok(())
}

#[cfg(test)]
mod test {
    use std::io::ErrorKind;

    use crate::{create_new_connection_handler, l, read_x_and_y_and_reply_with_sum, State};
    use tokio_test::io::Builder;

    #[tokio::test]
    async fn test_read_x_and_y_and_reply_with_sum() {
        let mut task_state = State::default();
        let mut buf = vec![0; 10];
        let mut event_receiver = l(&task_state).get_event_update_receiver();
        let mut socket = Builder::new()
            .write(b"< x = ")
            .read(b"3")
            .write(b"< y = ")
            .read(b"4")
            .write(b"> z = 7\n")
            .build();
        let r = read_x_and_y_and_reply_with_sum(
            &mut socket,
            &mut buf,
            &mut task_state,
            &mut event_receiver,
        )
        .await;
        assert!(r.is_ok());
    }

    #[tokio::test]
    async fn test_send_event() {
        let mut task_state = State::default();
        let mut buf = vec![0; 10];
        let mut event_receiver = l(&task_state).get_event_update_receiver();
        assert!(l(&task_state).send_event("blub").is_ok());
        let mut socket = Builder::new()
            .write(b"< x = ")
            .write(b"\n got event: blub\n")
            .read(b"3")
            .write(b"< y = ")
            .read(b"4")
            .write(b"> z = 7\n")
            .build();
        let r = read_x_and_y_and_reply_with_sum(
            &mut socket,
            &mut buf,
            &mut task_state,
            &mut event_receiver,
        )
        .await;
        assert!(r.is_ok());
    }

    #[tokio::test]
    async fn test_return_connection_aborted() {
        let mut task_state = State::default();
        let mut buf = vec![0; 10];
        let mut event_receiver = l(&task_state).get_event_update_receiver();
        let mut socket = Builder::new().write(b"< x = ").build();
        let r = read_x_and_y_and_reply_with_sum(
            &mut socket,
            &mut buf,
            &mut task_state,
            &mut event_receiver,
        )
        .await;
        assert!(r.is_err());
        let error = r.unwrap_err();
        assert_eq!(ErrorKind::ConnectionAborted, error.kind());
    }

    #[tokio::test]
    async fn test_create_new_connection_handler() {
        let task_state = State::default();
        let socket = Builder::new().write(b"< x = ").build();
        create_new_connection_handler(task_state)(socket);
    }
}
