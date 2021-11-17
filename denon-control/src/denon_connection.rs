use crate::parse::parse;
pub use crate::parse::{Operation, State};

use std::collections::HashSet;
use std::error::Error;
use std::io::{Read, Write};
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

fn read(stream: &mut dyn Read, lines: u8) -> Result<Vec<String>, std::io::Error> {
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

    loop {
        if let Ok((request, value)) = requests.try_recv() {
            if Operation::Stop == request {
                return Ok(());
            }
            let command;
            if Operation::Set == request {
                command = format!("{}\r", value);
            } else {
                command = format!("{}?\r", value.value());
            }
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
    return response
        .iter()
        .map(|x| parse(x.as_str()))
        .flatten()
        .collect();
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
    denon_name: String,
    denon_port: u16,
    state: Arc<Mutex<HashSet<State>>>,
    requests: Receiver<(Operation, State)>,
) {
    for _ in 0..10 {
        match TcpStream::connect((denon_name.as_str(), denon_port)) {
            Ok(s) => {
                match thread_func_impl(s, state, &requests) {
                    Ok(_) => println!("thread success"),
                    Err(e) => print_io_error(&e),
                }
                break;
            }
            Err(_) => {
                thread::sleep(Duration::from_millis(500));
            }
        }
    }
}

pub struct DenonConnection {
    state: Arc<Mutex<HashSet<State>>>,
    requests: Sender<(Operation, State)>,
}

impl DenonConnection {
    pub fn new(denon_name: &str, denon_port: u16) -> DenonConnection {
        let denon_string = String::from(denon_name);
        let state = Arc::new(Mutex::new(HashSet::new()));
        let cloned_state = state.clone();
        let (tx, rx) = channel();
        let _ = thread::spawn(move || {
            thread_func(denon_string, denon_port, cloned_state, rx);
        });
        DenonConnection {
            state,
            requests: tx,
        }
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
            thread::sleep(Duration::from_millis(100));
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
            thread::sleep(Duration::from_millis(100));
        }
    }
}
