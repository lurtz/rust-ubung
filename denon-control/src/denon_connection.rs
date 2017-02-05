use std::collections::HashMap;
use std::time::Duration;
use std::net::TcpStream;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::error::Error;
use std::io::{Read, Write};
use std::fmt::{Display, Formatter};

extern crate std;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Operation {
    MaxVolume,
    MainVolume,
    Power,
    SourceInput,
    Stop,
}

impl Operation {
    fn value(&self) -> &'static str {
        match *self {
            Operation::MaxVolume => "MVMAX",
            Operation::MainVolume => "MV",
            Operation::Power => "PW",
            Operation::SourceInput => "SI",
            Operation::Stop => "really stop now",
        }
    }

    fn parse(str: &str) -> Option<(Operation, State)> {
        let trimmed = str.trim().trim_matches('\r');
        if trimmed.starts_with(Operation::MaxVolume.value()) {
            let to_skip = Operation::MaxVolume.value().len();
            let ref to_parse = trimmed[to_skip..].trim();
            let mut value = to_parse.parse::<u32>().unwrap();
            if value < 100 {
                value = value * 10;
            }
            return Some((Operation::MaxVolume, State::Integer(value)));
        }
        if trimmed.starts_with(Operation::MainVolume.value()) {
            let to_skip = Operation::MainVolume.value().len();
            let ref to_parse = trimmed[to_skip..].trim();
            let mut value = to_parse.parse::<u32>().unwrap();
            if value < 100 {
                value = value * 10;
            }
            return Some((Operation::MainVolume, State::Integer(value)));
        }
        if trimmed.starts_with(Operation::Power.value()) {
            let to_skip = Operation::Power.value().len();
            let value = trimmed[to_skip..].to_string();
            return Some((Operation::Power, State::String(value)));
        }
        if trimmed.starts_with(Operation::SourceInput.value()) {
            let to_skip = Operation::SourceInput.value().len();
            let value = trimmed[to_skip..].to_string();
            return Some((Operation::SourceInput, State::String(value)));
        }
        None
    }
}

impl Display for Operation {
    fn fmt(&self, format: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(format, "{}", self.value())
    }
}

#[derive(Debug,Clone)]
pub enum State {
    Integer(u32),
    String(String),
}

impl Display for State {
    fn fmt(&self, format: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            &State::Integer(i) => write!(format, "{}", i),
            &State::String(ref s) => write!(format, "{}", s),
        }
    }
}

pub fn write(stream: &mut Write, input: String) -> Result<(), std::io::Error> {
    println!("sending: {}", input);
    let volume_command = input.into_bytes();
    stream.write(&volume_command[..])?;
    Ok(())
}

pub fn read(stream: &mut Read, lines: u8) -> Result<Vec<String>, std::io::Error> {
    let mut string = String::new();

    for _ in 0..lines {
        let mut buffer = [0; 100];
        let read_bytes = stream.read(&mut buffer)?;

        if let Ok(tmp) = std::str::from_utf8(&buffer[0..read_bytes]) {
            string += tmp;
        }
    }

    string.pop();

    let string_iter = string.split('\r').map(|x| String::from(x));
    let result = string_iter.collect();
    println!("{:?}", result);
    Ok(result)
}

fn thread_func_impl(denon_name: String,
                    denon_port: u16,
                    state: Arc<Mutex<HashMap<Operation, State>>>,
                    requests: Receiver<(Operation, State)>)
                    -> Result<(), std::io::Error> {
    let mut stream = TcpStream::connect((denon_name.as_str(), denon_port))?;
    stream.set_read_timeout(Some(Duration::from_secs(1)))?;

    loop {
        if let Ok((request, value)) = requests.try_recv() {
            if Operation::Stop == request {
                return Ok(());
            }
            let command = format!("{}{}\r", request, value);
            write(&mut stream, command)?;
        }

        match read(&mut stream, 1) {
            Ok(status_update) => {
                println!("received update {:?}", status_update);
                let parsed_response = parse_response(&status_update);
                let mut locked_state = state.lock().unwrap();
                for item in parsed_response {
                    locked_state.insert(item.0, item.1);
                }
            }
            // check for timeout error -> continue on timeout error, else abort
            Err(e) => {
                if std::io::ErrorKind::TimedOut != e.kind() &&
                   std::io::ErrorKind::WouldBlock != e.kind() {
                    return Err(e);
                }
            }
        }
    }
}

fn parse_response(response: &Vec<String>) -> Vec<(Operation, State)> {
    let mut result = Vec::new();
    for item in response {
        if let Some(parsed) = Operation::parse(item.as_str()) {
            result.push(parsed);
        }

    }
    result
}

pub fn print_io_error(e: &std::io::Error) {
    println!("got error: {}, cause = {:?}, description = {}, kind = {:?}",
             e,
             e.cause(),
             e.description(),
             e.kind());
    if let Some(raw_os_error) = e.raw_os_error() {
        println!("raw_os_error = {}", raw_os_error);
    }
    if let Some(inner) = e.get_ref() {
        println!("inner = {}", inner);
    }
}

fn thread_func(denon_name: String,
               denon_port: u16,
               state: Arc<Mutex<HashMap<Operation, State>>>,
               requests: Receiver<(Operation, State)>) {
    match thread_func_impl(denon_name, denon_port, state, requests) {
        Ok(_) => println!("thread success"),
        Err(e) => print_io_error(&e),
    }
}

pub struct DenonConnection {
    state: Arc<Mutex<HashMap<Operation, State>>>,
    requests: Sender<(Operation, State)>,
}

impl DenonConnection {
    pub fn new(denon_name: &str, denon_port: u16) -> DenonConnection {
        let denon_string = String::from(denon_name);
        let state = Arc::new(Mutex::new(HashMap::new()));
        let cloned_state = state.clone();
        let (tx, rx) = channel();
        thread::spawn(move || {
            thread_func(denon_string, denon_port, cloned_state, rx);
        });
        let dc = DenonConnection {
            state: state,
            requests: tx,
        };
        dc
    }

    pub fn get(&self, op: Operation) -> State {
        // should first check if the requested op is present in state
        // if it is not present it should send the request to the thread and wait until completion
        {
            let locked_state = self.state.lock().unwrap();
            if let Some(state) = locked_state.get(&op) {
                return state.clone();
            }
        }
        self.set(op.clone(), State::String(String::from("?")));
        for _ in 0..50 {
            thread::sleep(Duration::from_millis(100));
            let locked_state = self.state.lock().unwrap();
            if let Some(state) = locked_state.get(&op) {
                return state.clone();
            }
        }
        State::Integer(0)
    }

    pub fn set(&self, op: Operation, state: State) {
        self.requests.send((op.clone(), state)).unwrap();
    }
}

impl Drop for DenonConnection {
    fn drop(&mut self) {
        self.set(Operation::Stop, State::Integer(0));
    }
}
