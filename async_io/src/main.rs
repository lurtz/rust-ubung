use std::io::{self, ErrorKind};
#[cfg(not(test))]
use std::io::{Write, stdin, stdout};
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
#[cfg(not(test))]
use tokio::signal;
use tokio::sync::watch as Channel_type;
use tokio::task::JoinHandle;

#[cfg(test)]
use mockall::{automock, mock};

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

#[cfg_attr(test, automock)]
// TODO make it clonable
trait Stdio {
    fn print(&self, line: &str) -> io::Result<usize>;
    fn flush(&self) -> io::Result<usize>;
    fn read_line(&self, buffer: &mut String) -> io::Result<usize>;
}

#[cfg(not(test))]
#[derive(Default)]
struct StdioImpl {}

#[cfg(not(test))]
impl Stdio for StdioImpl {
    fn print(&self, line: &str) -> io::Result<usize> {
        stdout().write(line.as_bytes())
    }

    fn flush(&self) -> io::Result<usize> {
        stdout().flush().map(|_| 0)
    }

    fn read_line(&self, buffer: &mut String) -> io::Result<usize> {
        stdin().read_line(buffer)
    }
}

fn io_thread_main(thread_state: &mut State, stdio: &dyn Stdio) -> io::Result<()> {
    let mut buffer = String::new();
    buffer.reserve(10);
    loop {
        stdio.print("Enter event content: ")?;
        stdio.flush()?;
        stdio.read_line(&mut buffer)?;

        let _ = l(thread_state).send_event(buffer.trim());
        buffer.clear();
    }
}

fn create_new_connection_handler<Socket>(le_state: State) -> impl Fn(Socket) -> JoinHandle<()>
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
        })
    }
}

#[cfg_attr(test, automock)]
trait CtrlCWaiter {
    async fn ctrl_c_pressed(&self);
}

#[cfg(not(test))]
#[derive(Default)]
struct CtrlCWaiterImpl {}

#[cfg(not(test))]
impl CtrlCWaiter for CtrlCWaiterImpl {
    async fn ctrl_c_pressed(&self) {
        signal::ctrl_c().await.unwrap();
    }
}

#[cfg(test)]
mock! {
    pub AsyncMockCtrlWaiter {}
    impl Clone for AsyncMockCtrlWaiter {
        fn clone(&self) -> Self;
    }
    impl CtrlCWaiter for AsyncMockCtrlWaiter {
        // This implementation of the mock trait method is required to allow the mock methods to return a future.
        fn ctrl_c_pressed(
            &self,
        )-> impl std::future::Future<Output=()> + Send;
    }
}

async fn main2(
    listener: TcpListener,
    ctrl_c_waiter: &impl CtrlCWaiter,
    stdio: Box<dyn Stdio + Send>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("listening on {}", listener.local_addr()?);

    let le_state = State::default();

    // send event from user io thread, cannot be managed by tokio, because
    // reading from stdin blocks. Due to that the runtime will not shutdown
    // without user input. But with a normal OS thread the application
    // terminates as expected.
    let mut thread_state = le_state.clone();
    std::thread::spawn(move || {
        let _ = io_thread_main(&mut thread_state, stdio.as_ref());
    });

    let handle_new_connection = create_new_connection_handler(le_state);

    tokio::spawn(async move {
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            handle_new_connection(socket);
        }
    });

    ctrl_c_waiter.ctrl_c_pressed().await;
    println!("terminating");

    Ok(())
}

#[cfg(not(test))]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    main2(
        listener,
        &CtrlCWaiterImpl::default(),
        Box::new(StdioImpl::default()),
    )
    .await
}

#[cfg(test)]
mod test {
    use std::{
        io::{self, ErrorKind, Read, Write},
        net::TcpStream,
        ops::DerefMut,
        sync::{Arc, mpsc},
        thread::{self},
    };

    use crate::{
        MockAsyncMockCtrlWaiter, MockCtrlCWaiter, MockStdio, State, create_new_connection_handler,
        l, main2, read_x_and_y_and_reply_with_sum,
    };
    use mockall::predicate::eq;
    use tokio::{
        net::TcpListener,
        sync::{Mutex, oneshot},
    };
    use tokio_test::io::Builder;

    fn nothing() {}

    fn create_ctrl_c_mock() -> (MockAsyncMockCtrlWaiter, tokio::sync::oneshot::Sender<()>) {
        let (tx, rx) = oneshot::channel();
        let rx = Arc::new(Mutex::new(rx));
        let mut ctrl_c_mock = MockAsyncMockCtrlWaiter::new();
        ctrl_c_mock
            .expect_ctrl_c_pressed()
            .once()
            .returning(move || {
                let rxc = rx.clone();
                Box::pin(async move {
                    rxc.try_lock().unwrap().deref_mut().await.unwrap();
                    ()
                })
            });
        (ctrl_c_mock, tx)
    }

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
    async fn test_create_new_connection_handler_computes_result() {
        let task_state = State::default();
        let socket = Builder::new()
            .write(b"< x = ")
            .read(b"3")
            .write(b"< y = ")
            .read(b"4")
            .write(b"> z = 7\n")
            .write(b"< x = ")
            .build();
        assert!(
            create_new_connection_handler(task_state)(socket)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_create_new_connection_handler_aborts_connection() {
        let task_state = State::default();
        let socket = Builder::new().write(b"< x = ").build();
        assert!(
            create_new_connection_handler(task_state)(socket)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_main_terminates_when_ctrl_pressed() {
        let mut ctrl_c_mock = MockCtrlCWaiter::new();
        ctrl_c_mock
            .expect_ctrl_c_pressed()
            .once()
            .returning(nothing);
        let mut stdio_mock = Box::new(MockStdio::default());
        stdio_mock
            .expect_print()
            .once()
            .returning(|_| Err(io::Error::new(ErrorKind::BrokenPipe, "")));

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let _mr = main2(listener, &ctrl_c_mock, stdio_mock).await;
    }

    #[tokio::test]
    async fn test_main_accepts_connection() {
        let (ctrl_c_mock, tx) = create_ctrl_c_mock();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_address = listener.local_addr().unwrap();
        let response = thread::spawn(move || {
            let mut to_server = TcpStream::connect(local_address).unwrap();
            let mut buf = [0; 10];
            to_server.read_exact(&mut buf[0..6]).unwrap();
            assert_eq!("< x = ", std::str::from_utf8(&buf[0..6]).unwrap());
            to_server.write_all(b"7\n").unwrap();
            to_server.read_exact(&mut buf[0..6]).unwrap();
            assert_eq!("< y = ", std::str::from_utf8(&buf[0..6]).unwrap());
            to_server.write_all(b"3\n").unwrap();
            to_server.read_exact(&mut buf[0..9]).unwrap();
            tx.send(()).unwrap();
            buf
        });
        let mut stdio_mock = Box::new(MockStdio::new());
        stdio_mock
            .expect_print()
            .with(eq("Enter event content: "))
            .returning(|_a| Ok(0));
        stdio_mock.expect_flush().once().returning(|| Ok(0));
        let (tx, rx) = mpsc::channel();
        stdio_mock.expect_read_line().once().returning(move |_| {
            rx.recv().unwrap();
            Err(io::Error::new(io::ErrorKind::BrokenPipe, ""))
        });
        let _mr = main2(listener, &ctrl_c_mock, stdio_mock).await;
        tx.send(()).unwrap();
        _mr.unwrap();
        let result = response.join().unwrap();
        assert_eq!("> z = 10\n", std::str::from_utf8(&result[0..9]).unwrap());
    }

    #[tokio::test]
    async fn test_main_sends_event() {
        let (ctrl_c_mock, tx) = create_ctrl_c_mock();
        let (set_connected, is_connected) = mpsc::channel();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_address = listener.local_addr().unwrap();
        let response = thread::spawn(move || {
            let mut to_server = TcpStream::connect(local_address).unwrap();
            let mut buf = [0; 20];
            to_server.read_exact(&mut buf[0..6]).unwrap();
            set_connected.send(()).unwrap();
            assert_eq!("< x = ", std::str::from_utf8(&buf[0..6]).unwrap());
            to_server.read_exact(&mut buf[0..18]).unwrap();
            tx.send(()).unwrap();
            buf
        });
        let mut stdio_mock = Box::new(MockStdio::new());
        stdio_mock
            .expect_print()
            .once()
            .with(eq("Enter event content: "))
            .returning(|_a| Ok(0));
        stdio_mock.expect_flush().once().returning(|| Ok(0));
        stdio_mock
            .expect_read_line()
            .once()
            .returning(move |buf: &mut String| {
                is_connected.recv().unwrap();
                buf.clear();
                buf.reserve(4);
                buf.push_str("blub");
                Ok(buf.len())
            });
        let (tx, rx) = mpsc::channel();
        stdio_mock
            .expect_print()
            .once()
            .with(eq("Enter event content: "))
            .returning(move |_| {
                rx.recv().unwrap();
                Err(io::Error::new(io::ErrorKind::BrokenPipe, ""))
            });
        let _mr = main2(listener, &ctrl_c_mock, stdio_mock).await;
        tx.send(()).unwrap();
        _mr.unwrap();
        let result = response.join().unwrap();
        assert_eq!(
            "\n got event: blub\n",
            std::str::from_utf8(&result[0..18]).unwrap()
        );
    }
}
