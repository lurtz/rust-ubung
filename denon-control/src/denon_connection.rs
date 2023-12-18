use crate::parse::parse;
pub use crate::parse::{Operation, State};

use std::collections::HashSet;
use std::error::Error;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn write(stream: &mut dyn Write, input: String) -> Result<(), std::io::Error> {
    //    println!("sending: {}", input);
    let volume_command = input.into_bytes();
    stream.write_all(&volume_command[..])?;
    Ok(())
}

pub fn read(stream: &mut dyn Read, lines: u8) -> Result<Vec<String>, std::io::Error> {
    let mut string = String::new();

    for _ in 0..lines {
        let mut buffer = [0; 100];
        let read_bytes = stream.read(&mut buffer)?;
        if let Ok(tmp) = std::str::from_utf8(&buffer[0..read_bytes]) {
            string += tmp;
        }
    }

    string.pop();

    let string_iter = string.split('\r').map(String::from);
    let result = string_iter.collect();
    //    println!("{:?}", result);
    Ok(result)
}

fn thread_func_impl(
    mut stream: TcpStream,
    state: Arc<Mutex<HashSet<State>>>,
    requests: &Receiver<(Operation, State)>,
) -> Result<(), std::io::Error> {
    stream.set_read_timeout(Some(Duration::from_secs(1)))?;

    // https://docs.rs/polling/latest/polling/
    // maybe use poll() instead of this
    loop {
        if let Ok((request, value)) = requests.try_recv() {
            if Operation::Stop == request {
                return Ok(());
            }
            let command = if Operation::Set == request {
                format!("{}\r", value)
            } else {
                format!("{}?\r", value.value())
            };
            write(&mut stream, command)?;
        }

        match read(&mut stream, 1) {
            Ok(status_update) => {
                //                println!("received update {:?}", status_update);
                let parsed_response = parse_response(&status_update);
                let mut locked_state = state.lock().unwrap();
                for item in parsed_response {
                    locked_state.replace(item);
                }
            }
            // check for timeout error -> continue on timeout error, else abort
            Err(e) => {
                if std::io::ErrorKind::TimedOut != e.kind()
                    && std::io::ErrorKind::WouldBlock != e.kind()
                {
                    return Err(e);
                }
            }
        }
    }
}

fn parse_response(response: &[String]) -> Vec<State> {
    return response.iter().filter_map(|x| parse(x.as_str())).collect();
}

fn print_io_error(e: &std::io::Error) {
    println!(
        "got error: {}, source = {:?}, description = {}, kind = {:?}",
        e,
        e.source(),
        e,
        e.kind()
    );
    if let Some(raw_os_error) = e.raw_os_error() {
        println!("raw_os_error = {}", raw_os_error);
    }
    if let Some(inner) = e.get_ref() {
        println!("inner = {}", inner);
    }
}

fn thread_func(
    s: TcpStream,
    state: Arc<Mutex<HashSet<State>>>,
    requests: Receiver<(Operation, State)>,
) {
    match thread_func_impl(s, state, &requests) {
        Ok(_) => println!("thread success"),
        Err(e) => print_io_error(&e),
    }
}

pub struct DenonConnection {
    state: Arc<Mutex<HashSet<State>>>,
    requests: Sender<(Operation, State)>,
}

impl DenonConnection {
    pub fn new(denon_name: String, denon_port: u16) -> Result<DenonConnection, io::Error> {
        let state = Arc::new(Mutex::new(HashSet::new()));
        let cloned_state = state.clone();
        let (tx, rx) = channel();
        let s = TcpStream::connect((denon_name.as_str(), denon_port))?;
        let _ = thread::spawn(move || {
            thread_func(s, cloned_state, rx);
        });

        Ok(DenonConnection {
            state,
            requests: tx,
        })
    }

    pub fn get(&self, op: State) -> Result<State, std::sync::mpsc::SendError<(Operation, State)>> {
        // should first check if the requested op is present in state
        // if it is not present it should send the request to the thread and wait until completion
        {
            let locked_state = self.state.lock().unwrap();
            if let Some(received_state) = locked_state.get(&op) {
                return Ok(received_state.clone());
            }
        }
        self.query(op.clone(), Operation::Query)?;
        for _ in 0..50 {
            thread::sleep(Duration::from_millis(10));
            let locked_state = self.state.lock().unwrap();
            if let Some(state) = locked_state.get(&op) {
                return Ok(state.clone());
            }
        }
        Ok(State::Unknown)
    }

    fn query(
        &self,
        state: State,
        op: Operation,
    ) -> Result<(), std::sync::mpsc::SendError<(Operation, State)>> {
        self.requests.send((op, state))
    }

    pub fn stop(&self) -> Result<(), std::sync::mpsc::SendError<(Operation, State)>> {
        self.query(State::Unknown, Operation::Stop)
    }

    pub fn set(&self, state: State) -> Result<(), std::sync::mpsc::SendError<(Operation, State)>> {
        self.query(state, Operation::Set)
    }
}

impl Drop for DenonConnection {
    fn drop(&mut self) {
        while Ok(()) == self.stop() {
            thread::sleep(Duration::from_millis(10));
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::DenonConnection;
    use crate::denon_connection::{read, write};
    use crate::operation::Operation;
    use crate::state::State;
    use std::io;
    use std::net::{TcpListener, TcpStream};
    use std::sync::mpsc::SendError;

    pub fn create_connected_connection() -> Result<(DenonConnection, TcpStream), io::Error> {
        let listen_socket = TcpListener::bind("127.0.0.1:0")?;
        let addr = listen_socket.local_addr()?;
        let dc = DenonConnection::new(addr.ip().to_string(), addr.port())?;
        let (to_denon_client, _) = listen_socket.accept()?;
        Ok((dc, to_denon_client))
    }

    #[test]
    fn fails_to_connect_and_returns_unknown() {
        let dc = DenonConnection::new(String::from("value"), 0);
        assert!(matches!(dc, Err(_)));
    }

    #[test]
    fn connection_gets_no_reply_and_returns_unknown() -> Result<(), io::Error> {
        let (dc, mut to_denon_client) = create_connected_connection()?;
        let rc = dc.get(State::main_volume());
        let query = read(&mut to_denon_client, 1)?;
        let x: Result<State, SendError<(Operation, State)>> = Ok(State::Unknown);
        assert_eq!(rc, x);
        assert_eq!(query, vec!["MV?"]);
        Ok(())
    }

    #[test]
    fn connection_sends_volume_to_receiver() -> Result<(), io::Error> {
        let (dc, mut to_denon_client) = create_connected_connection()?;
        dc.set(State::MainVolume(666)).unwrap();
        let received = read(&mut to_denon_client, 1)?;
        assert_eq!("MV666", received[0]);
        Ok(())
    }

    #[test]
    fn connection_receives_volume_from_receiver() -> Result<(), io::Error> {
        let (dc, mut to_denon_client) = create_connected_connection()?;
        write(&mut to_denon_client, "MV234".to_string())?;
        assert_eq!(
            State::MainVolume(234),
            dc.get(State::MainVolume(666)).unwrap()
        );
        Ok(())
    }

    #[test]
    fn connection_keeps_first_after_second_receive() -> Result<(), io::Error> {
        let (dc, mut to_denon_client) = create_connected_connection()?;
        write(&mut to_denon_client, "MV234".to_string())?;
        assert_eq!(
            State::MainVolume(234),
            dc.get(State::MainVolume(666)).unwrap()
        );
        write(&mut to_denon_client, "MV320".to_string())?;
        assert_eq!(
            State::MainVolume(234),
            dc.get(State::MainVolume(666)).unwrap()
        );
        Ok(())
    }
}
