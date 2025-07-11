use std::io::{self, ErrorKind};

use bytes::{Buf, BufMut, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufStream};
use tokio::net::TcpListener;
use tokio::sync::watch as Channel_type;
use tokio::task::JoinHandle;

use crate::ctrl_c_waiter::CtrlCWaiter;
use crate::state::State;
use crate::stdio::Stdio;

#[cfg(test)]
use mockall::mock;

async fn read_int<Reader, Buff>(
    socket: &mut Reader,
    buf: &mut Buff,
) -> Result<usize, std::io::Error>
where
    Reader: AsyncReadExt + Unpin,
    Buff: BufMut + Buf + Send,
{
    loop {
        let n = socket.read_buf(buf).await?;
        if 0 == n {
            return Err(std::io::Error::from(ErrorKind::ConnectionAborted));
        }
        // check for \n character
        let parsed_string = std::str::from_utf8(buf.chunk()).unwrap();
        if let Some(pos) = parsed_string.find('\n') {
            let x: usize = parsed_string[0..pos].trim().parse().unwrap();
            buf.advance(pos + 1);
            return Ok(x);
        }
    }
}

struct Connection<Socket>
where
    Socket: AsyncReadExt + AsyncWriteExt + Unpin + Send + 'static,
{
    buf: BytesMut,
    task_state: State,
    event_receiver: Channel_type::Receiver<String>,
    socket: BufStream<Socket>,
}

impl<Socket> Connection<Socket>
where
    Socket: AsyncReadExt + AsyncWriteExt + Unpin + Send + 'static,
{
    fn new(task_state: State, socket: Socket) -> Connection<Socket> {
        let event_receiver = task_state.get_event_update_receiver();
        Connection {
            buf: BytesMut::with_capacity(10),
            task_state,
            event_receiver,
            socket: BufStream::new(socket),
        }
    }

    async fn read_int_and_watch_for_event(&mut self) -> Result<usize, std::io::Error> {
        let n;
        loop {
            tokio::select! {
                x = read_int(&mut self.socket, &mut self.buf) => {n=x?; break;},
                _ = self.event_receiver.changed() => {
                    let event_payload = format!(
                        "\n got event: {}\n",
                        *self.event_receiver.borrow_and_update()
                    );
                    self.socket
                        .write_all(event_payload.as_bytes())
                        .await?;
                    self.socket.flush().await?;
                }
            };
        }
        Ok(n)
    }

    async fn read_x_and_y_and_reply_with_sum(&mut self) -> Result<(), std::io::Error> {
        {
            self.socket.write_all("< x = ".as_bytes()).await?;
            self.socket.flush().await?;
            let x = self.read_int_and_watch_for_event().await?;
            self.task_state.set_x(x);
        }
        {
            self.socket.write_all("< y = ".as_bytes()).await?;
            self.socket.flush().await?;
            let y = self.read_int_and_watch_for_event().await?;
            self.task_state.set_y(y);
        }

        // Write the data back
        let z = self.task_state.get_z();
        self.socket
            .write_all(format!("> z = {z}\n").as_bytes())
            .await?;
        self.socket.flush().await?;

        Ok(())
    }
}

fn create_new_connection_handler<Socket>(
    le_state: State,
) -> impl Fn(Socket) -> JoinHandle<Result<(), std::io::Error>>
where
    Socket: AsyncReadExt + AsyncWriteExt + Unpin + Send + 'static,
{
    move |socket| {
        let mut task_state = le_state.clone();
        println!("new connection {}", task_state.inc_counter());
        let mut connection = Connection::new(task_state, socket);

        tokio::spawn(async move {
            // In a loop, read data from the socket and write the data back.
            loop {
                if let Err(e) = connection.read_x_and_y_and_reply_with_sum().await {
                    eprintln!("socket failure; err = {e:?}");
                    return Err(e);
                }
            }
        })
    }
}

pub trait MyTcpListener {
    type Stream: Unpin + Send + AsyncReadExt + AsyncWriteExt;

    fn local_addr(&self) -> io::Result<std::net::SocketAddr>;
    fn accept(
        &self,
    ) -> impl std::future::Future<Output = io::Result<(Self::Stream, std::net::SocketAddr)>> + Send;
}

impl MyTcpListener for TcpListener {
    type Stream = tokio::net::TcpStream;

    fn local_addr(&self) -> io::Result<std::net::SocketAddr> {
        self.local_addr()
    }

    fn accept(
        &self,
    ) -> impl std::future::Future<Output = io::Result<(Self::Stream, std::net::SocketAddr)>> + Send
    {
        self.accept()
    }
}

#[cfg(test)]
mock! {
    pub MyTcpListenerMock {}

    impl MyTcpListener for MyTcpListenerMock {
        type Stream = tokio_test::io::Mock;
        // This implementation of the mock trait method is required to allow the mock methods to return a future.
        fn local_addr(&self) -> io::Result<std::net::SocketAddr>;
        fn accept(
            &self,
        ) -> impl std::future::Future<Output = io::Result<(tokio_test::io::Mock, std::net::SocketAddr)>> + Send;
    }
}

fn io_thread_main(thread_state: &mut State, stdio: &dyn Stdio) -> io::Result<()> {
    let mut buffer = String::new();
    buffer.reserve(10);
    loop {
        stdio.print("Enter event content: ")?;
        stdio.flush()?;
        stdio.read_line(&mut buffer)?;

        let _ = thread_state.send_event(buffer.trim());
        buffer.clear();
    }
}

pub async fn main2<Listener>(
    listener: Listener,
    ctrl_c_waiter: &impl CtrlCWaiter,
    stdio: Box<dyn Stdio + Send>,
) -> Result<(), Box<dyn std::error::Error>>
where
    Listener: MyTcpListener + Send + 'static,
{
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
            let accept_result = listener.accept().await;
            if let Ok((socket, _)) = accept_result {
                handle_new_connection(socket);
            }
        }
    });

    ctrl_c_waiter.ctrl_c_pressed().await;
    println!("terminating");

    Ok(())
}

#[cfg(test)]
mod test {
    use std::{
        io::{self, ErrorKind, Read, Write},
        mem::swap,
        net::{SocketAddr, TcpStream},
        ops::DerefMut,
        str::FromStr,
        sync::{Arc, mpsc},
        thread::{self, sleep},
        time::Duration,
    };

    use crate::async_adder::{MockMyTcpListenerMock, State, create_new_connection_handler, main2};
    use crate::ctrl_c_waiter::MockAsyncMockCtrlWaiter;
    use crate::stdio::MockStdio;
    use mockall::predicate::eq;
    use tokio::{
        net::TcpListener,
        sync::{Mutex, oneshot},
    };
    use tokio_test::io::Builder;

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
                    rxc.lock().await.deref_mut().await.unwrap();
                })
            });
        (ctrl_c_mock, tx)
    }

    fn create_listener_mock() -> MockMyTcpListenerMock {
        let mut listener_mock = MockMyTcpListenerMock::new();
        listener_mock
            .expect_local_addr()
            .returning(|| Ok(SocketAddr::from_str("127.0.0.1:1234").unwrap()));
        listener_mock
    }

    fn setup_last_accept(
        listener_mock: &mut MockMyTcpListenerMock,
        terminate_main2: oneshot::Sender<()>,
    ) {
        let terminate_main2 = Arc::new(Mutex::new(terminate_main2));
        listener_mock.expect_accept().once().returning(move || {
            let txx = terminate_main2.clone();
            Box::pin(async move {
                let (mut tx2, _) = oneshot::channel::<()>();
                swap(&mut tx2, txx.lock().await.deref_mut());
                tx2.send(()).unwrap();
                let (_keep_sender_alive, rx) = oneshot::channel();
                // rx will never return
                let le_error = io::Error::new(io::ErrorKind::BrokenPipe, "");
                rx.await.map_err(|_| le_error)
            })
        });
    }

    fn create_blocked_io_mock() -> (Box<MockStdio>, mpsc::Sender<()>) {
        let (tx2, rx) = mpsc::channel();
        let mut stdio_mock = Box::new(MockStdio::new());
        stdio_mock
            .expect_print()
            .with(eq("Enter event content: "))
            .returning(move |_| {
                rx.recv().unwrap();
                Err(io::Error::new(io::ErrorKind::BrokenPipe, ""))
            });
        (stdio_mock, tx2)
    }

    #[tokio::test]
    async fn test_return_connection_aborted() {
        let task_state = State::default();
        let socket = Builder::new().write(b"< x = ").build();
        let join_result = create_new_connection_handler(task_state)(socket).await;
        assert!(join_result.is_ok());
        let r = join_result.unwrap();
        assert!(r.is_err());
        let error = r.unwrap_err();
        assert_eq!(ErrorKind::ConnectionAborted, error.kind());
    }

    #[tokio::test]
    async fn test_newline_triggers_number_parsing() {
        let task_state = State::default();
        let socket = Builder::new()
            .write(b"< x = ")
            .read(b"3")
            .read(b"4")
            .read(b"5")
            .read(b"6")
            .read(b"\n")
            .write(b"< y = ")
            .read(b"4\n")
            .write(b"> z = 3460\n")
            .write(b"< x = ")
            .build();
        assert!(
            create_new_connection_handler(task_state)(socket)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_main_terminates_when_ctrl_pressed() {
        let (ctrl_c_mock, terminate_main2) = create_ctrl_c_mock();
        let mut stdio_mock = Box::new(MockStdio::default());
        stdio_mock
            .expect_print()
            .once()
            .returning(|_| Err(io::Error::new(ErrorKind::BrokenPipe, "")));

        let mut listener_mock = create_listener_mock();
        setup_last_accept(&mut listener_mock, terminate_main2);
        let _mr = main2(listener_mock, &ctrl_c_mock, stdio_mock).await;
        assert!(_mr.is_ok());
    }

    #[tokio::test]
    async fn test_main_accepts_connection() {
        let (ctrl_c_mock, terminate_main2) = create_ctrl_c_mock();
        let mut listener_mock = create_listener_mock();

        listener_mock.expect_accept().once().returning(move || {
            Box::pin(async move {
                let socket_mock = Builder::new().write(b"< x = ").build();
                Ok((socket_mock, SocketAddr::from_str("127.0.0.1:1234").unwrap()))
            })
        });

        setup_last_accept(&mut listener_mock, terminate_main2);
        let (stdio_mock, tx2) = create_blocked_io_mock();
        let _mr = main2(listener_mock, &ctrl_c_mock, stdio_mock).await;
        tx2.send(()).unwrap();
        _mr.unwrap();
    }

    #[tokio::test]
    async fn test_main_ignores_accept_error() {
        let (ctrl_c_mock, terminate_main2) = create_ctrl_c_mock();
        let mut listener_mock = create_listener_mock();

        listener_mock.expect_accept().once().returning(move || {
            Box::pin(async move { Err(io::Error::new(io::ErrorKind::BrokenPipe, "")) })
        });

        setup_last_accept(&mut listener_mock, terminate_main2);
        let (stdio_mock, tx2) = create_blocked_io_mock();
        let _mr = main2(listener_mock, &ctrl_c_mock, stdio_mock).await;
        tx2.send(()).unwrap();
        _mr.unwrap();
    }

    #[tokio::test]
    async fn test_main_computes_result() {
        let (ctrl_c_mock, tx) = create_ctrl_c_mock();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_address = listener.local_addr().unwrap();
        let response = thread::spawn(move || {
            let mut to_server = TcpStream::connect(local_address).unwrap();
            let mut buf = [0; 10];
            to_server.read_exact(&mut buf[0..6]).unwrap();
            assert_eq!("< x = ", std::str::from_utf8(&buf[0..6]).unwrap());
            to_server.write_all(b"23\n").unwrap();
            buf = [0; 10];
            to_server.read_exact(&mut buf[0..6]).unwrap();
            assert_eq!("< y = ", std::str::from_utf8(&buf[0..6]).unwrap());
            to_server.write_all(b"2\n").unwrap();
            buf = [0; 10];
            to_server.read_exact(&mut buf[0..9]).unwrap();
            tx.send(()).unwrap();
            buf
        });
        let mut stdio_mock = Box::new(MockStdio::new());
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
        assert_eq!("> z = 25\n", std::str::from_utf8(&result[0..9]).unwrap());
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

    #[tokio::test]
    async fn test_main_sends_event_with_more_mocks_but_unstable() {
        let (ctrl_c_mock, mut terminate_main2) = create_ctrl_c_mock();
        let mut listener_mock = create_listener_mock();

        listener_mock.expect_accept().once().returning(move || {
            Box::pin(async move {
                println!("expected accept called");
                let socket_mock = Builder::new()
                    .write(b"< x = ")
                    .write(b"\n got event: blub\n")
                    .build();
                Ok((socket_mock, SocketAddr::from_str("127.0.0.1:1234").unwrap()))
            })
        });

        let (set_connected, mut is_connected) = oneshot::channel();
        setup_last_accept(&mut listener_mock, set_connected);

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
                let (_, mut is_connected2) = oneshot::channel();
                swap(&mut is_connected, &mut is_connected2);
                is_connected2.blocking_recv().unwrap();
                buf.clear();
                buf.reserve(4);
                buf.push_str("blub");
                Ok(buf.len())
            });
        stdio_mock
            .expect_print()
            .once()
            .with(eq("Enter event content: "))
            .returning(move |_| {
                let (mut terminate_main, _) = oneshot::channel();
                swap(&mut terminate_main, &mut terminate_main2);
                // this sleep makes the test unstable compare to the previous
                // test. There is no API to check if the mock created in the
                // first mocked accept() had all its expectations fullfilled or
                // place some action when all its expectations are fulfilled.
                sleep(Duration::from_millis(100));
                terminate_main.send(()).unwrap();
                Err(io::Error::new(io::ErrorKind::BrokenPipe, ""))
            });

        let _mr = main2(listener_mock, &ctrl_c_mock, stdio_mock).await;
        _mr.unwrap();
    }
}
