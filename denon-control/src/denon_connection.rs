use crate::parse::parse;
pub use crate::parse::{Operation, State};
use crate::state::{SetState, StateValue};

use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::panic;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const ESHUTDOWN: i32 = 108;

fn write_string(stream: &mut dyn Write, input: String) -> Result<(), std::io::Error> {
    let volume_command = input.into_bytes();
    stream.write_all(&volume_command[..])?;
    Ok(())
}

pub fn write(
    stream: &mut dyn Write,
    state: State,
    value: StateValue,
    op: Operation,
) -> Result<(), io::Error> {
    let command = if Operation::Set == op {
        format!("{}{}\r", state, value)
    } else {
        format!("{}?\r", state.value())
    };
    write_string(stream, command)
}

pub fn read(mut stream: &TcpStream, lines: u8) -> Result<Vec<String>, std::io::Error> {
    let mut result = Vec::<String>::new();

    // guarantee to read a full line. check that read content ends with \r
    while (lines as usize) != result.len() {
        let mut buffer = [0; 100];
        let read_bytes;
        match stream.peek(&mut buffer) {
            Ok(rb) => read_bytes = rb,
            Err(e) => {
                if result.is_empty() {
                    return Err(e);
                } else {
                    break;
                }
            }
        }

        if 0 == read_bytes {
            // shutdown called
            if result.is_empty() {
                return Err(io::Error::from_raw_os_error(ESHUTDOWN));
            } else {
                break;
            }
        }

        // search for first \r in buffer
        let first_cariage_return = buffer[0..read_bytes]
            .iter()
            .position(|&c| '\r' == (c as char));

        if first_cariage_return.is_none() {
            break;
        }

        // include cariage return in read_exact()
        let bytes_to_extract = first_cariage_return.unwrap() + 1;

        // do not add \r to string
        if let Ok(tmp) = std::str::from_utf8(&buffer[0..first_cariage_return.unwrap()]) {
            result.push(tmp.trim().to_owned());
        }

        stream.read_exact(&mut buffer[0..bytes_to_extract])?;
    }

    Ok(result)
}

fn thread_func_impl(
    stream: &TcpStream,
    state: Arc<Mutex<HashMap<State, StateValue>>>,
) -> Result<(), std::io::Error> {
    loop {
        match read(stream, 1) {
            Ok(status_update) => {
                let parsed_response = parse_response(&status_update);
                let mut locked_state = state.lock().unwrap();
                for (state, value) in parsed_response {
                    locked_state.insert(state, value);
                }
            }
            // check for timeout error -> continue on timeout error, else abort
            Err(e) => {
                if std::io::ErrorKind::TimedOut != e.kind()
                    && std::io::ErrorKind::WouldBlock != e.kind()
                {
                    if e.raw_os_error() != Some(ESHUTDOWN) {
                        return Err(e);
                    }
                    return Ok(());
                }
            }
        }
    }
}

fn parse_response(response: &[String]) -> Vec<(State, StateValue)> {
    return response.iter().filter_map(|x| parse(x.as_str())).collect();
}

pub struct DenonConnection {
    state: Arc<Mutex<HashMap<State, StateValue>>>,
    to_receiver: TcpStream,
    thread_handle: Option<JoinHandle<Result<(), io::Error>>>,
}

impl DenonConnection {
    pub fn new(denon_name: String, denon_port: u16) -> Result<DenonConnection, io::Error> {
        let state = Arc::new(Mutex::new(HashMap::new()));
        let cloned_state = state.clone();
        let s = TcpStream::connect((denon_name.as_str(), denon_port))?;
        let read_timeout = None;
        s.set_read_timeout(read_timeout)?;
        s.set_nonblocking(false)?;
        let s2 = s.try_clone()?;

        let threadhandle = thread::spawn(move || thread_func_impl(&s2, cloned_state));

        Ok(DenonConnection {
            state,
            to_receiver: s,
            thread_handle: Some(threadhandle),
        })
    }

    pub fn get(&mut self, op: State) -> Result<StateValue, io::Error> {
        // should first check if the requested op is present in state
        // if it is not present it should send the request to the thread and wait until completion
        {
            let locked_state = self.state.lock().unwrap();
            if let Some(received_state) = locked_state.get(&op) {
                return Ok(*received_state);
            }
        }
        self.query(op, StateValue::Unknown, Operation::Query)?;
        for _ in 0..50 {
            thread::sleep(Duration::from_millis(10));
            let locked_state = self.state.lock().unwrap();
            if let Some(state) = locked_state.get(&op) {
                return Ok(*state);
            }
        }
        Ok(StateValue::Unknown)
    }

    fn query(&mut self, state: State, value: StateValue, op: Operation) -> Result<(), io::Error> {
        write(&mut self.to_receiver, state, value, op)
    }

    pub fn stop(&mut self) -> Result<(), io::Error> {
        self.to_receiver.shutdown(std::net::Shutdown::Both)
    }

    pub fn set(&mut self, state: SetState) -> Result<(), io::Error> {
        match state {
            SetState::MainVolume(i) => {
                self.query(State::MainVolume, StateValue::Integer(i), Operation::Set)
            }

            SetState::MaxVolume(i) => {
                self.query(State::MaxVolume, StateValue::Integer(i), Operation::Set)
            }
            SetState::Power(ps) => self.query(State::Power, StateValue::Power(ps), Operation::Set),
            SetState::SourceInput(si) => self.query(
                State::SourceInput,
                StateValue::SourceInput(si),
                Operation::Set,
            ),
        }
    }
}

impl Drop for DenonConnection {
    fn drop(&mut self) {
        let _ = self.stop();
        let thread_result = self
            .thread_handle
            .take()
            .expect("Non running thread is a bug")
            .join();
        match thread_result {
            Ok(result) => {
                if let Err(e) = result {
                    // TODO only one test should trigger this
                    println!("got error: {}", e)
                }
            }
            Err(e) => panic::resume_unwind(e),
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::DenonConnection;
    use crate::denon_connection::{read, write_string};
    use crate::parse::PowerState;
    use crate::parse::SourceInputState;
    use crate::state::{SetState, State, StateValue};
    use std::io;
    use std::net::{TcpListener, TcpStream};
    use std::thread::yield_now;

    pub fn create_connected_connection() -> Result<(TcpStream, DenonConnection), io::Error> {
        let listen_socket = TcpListener::bind("127.0.0.1:0")?;
        let addr = listen_socket.local_addr()?;
        let dc = DenonConnection::new(addr.ip().to_string(), addr.port())?;
        let (to_denon_client, _) = listen_socket.accept()?;
        Ok((to_denon_client, dc))
    }

    macro_rules! wait_for_value_in_database {
        ($denon_connection:ident, $state:expr, $exp_state:pat) => {
            for _ in 0..100000 {
                if matches!($denon_connection.get($state)?, $exp_state) {
                    break;
                }
                yield_now();
            }
        };
    }

    macro_rules! assert_db_value {
        ($denon_connection:ident, $state:expr, $exp_state:pat) => {
            assert!(matches!($denon_connection.get($state)?, $exp_state));
        };
    }

    #[test]
    fn fails_to_connect_and_returns_unknown() {
        let dc = DenonConnection::new(String::from("value"), 0);
        assert!(matches!(dc, Err(_)));
    }

    #[test]
    fn connection_gets_no_reply_and_returns_unknown() -> Result<(), io::Error> {
        let (mut to_denon_client, mut dc) = create_connected_connection()?;
        let rc = dc.get(State::MainVolume)?;
        let query = read(&mut to_denon_client, 1)?;
        assert_eq!(rc, StateValue::Unknown);
        assert_eq!(query, vec!["MV?"]);
        Ok(())
    }

    #[test]
    fn connection_sends_main_volume_to_receiver() -> Result<(), io::Error> {
        let (mut to_denon_client, mut dc) = create_connected_connection()?;
        dc.set(SetState::MainVolume(666))?;
        let received = read(&mut to_denon_client, 1)?;
        assert_eq!("MV666", received[0]);
        Ok(())
    }

    #[test]
    fn connection_sends_max_volume_to_receiver() -> Result<(), io::Error> {
        let (mut to_denon_client, mut dc) = create_connected_connection()?;
        dc.set(SetState::MaxVolume(666))?;
        let received = read(&mut to_denon_client, 1)?;
        assert_eq!("MVMAX666", received[0]);
        Ok(())
    }

    #[test]
    fn connection_sends_source_input_to_receiver() -> Result<(), io::Error> {
        let (mut to_denon_client, mut dc) = create_connected_connection()?;
        dc.set(SetState::SourceInput(SourceInputState::Fvp))?;
        let received = read(&mut to_denon_client, 1)?;
        assert_eq!("SIFVP", received[0]);
        Ok(())
    }

    #[test]
    fn connection_sends_power_to_receiver() -> Result<(), io::Error> {
        let (mut to_denon_client, mut dc) = create_connected_connection()?;
        dc.set(SetState::Power(PowerState::On))?;
        let received = read(&mut to_denon_client, 1)?;
        assert_eq!("PWON", received[0]);
        Ok(())
    }

    #[test]
    fn connection_receives_volume_from_receiver() -> Result<(), io::Error> {
        let (mut to_denon_client, mut dc) = create_connected_connection()?;
        write_string(&mut to_denon_client, "MV234\r".to_string())?;
        assert_db_value!(dc, State::MainVolume, StateValue::Integer(234));
        Ok(())
    }

    #[test]
    fn connection_receives_multiple_values_volume_from_receiver() -> Result<(), io::Error> {
        let (mut to_denon_client, mut dc) = create_connected_connection()?;
        assert_eq!(StateValue::Unknown, dc.get(State::MainVolume)?);
        assert_eq!(StateValue::Unknown, dc.get(State::SourceInput)?);
        assert_eq!(StateValue::Unknown, dc.get(State::Power)?);
        write_string(&mut to_denon_client, "MV234\rSICD\rPWON\r".to_string())?;
        assert_db_value!(dc, State::MainVolume, StateValue::Integer(234));
        assert_db_value!(
            dc,
            State::SourceInput,
            StateValue::SourceInput(SourceInputState::Cd)
        );
        assert_db_value!(dc, State::Power, StateValue::Power(PowerState::On));
        Ok(())
    }

    #[test]
    fn connection_updates_values_with_newly_received_data() -> Result<(), io::Error> {
        let (mut to_denon_client, mut dc) = create_connected_connection()?;
        write_string(&mut to_denon_client, "MV234\r".to_string())?;
        assert_db_value!(dc, State::MainVolume, StateValue::Integer(234));
        write_string(&mut to_denon_client, "MV320\r".to_string())?;
        wait_for_value_in_database!(dc, State::MainVolume, StateValue::Integer(320));
        assert_db_value!(dc, State::MainVolume, StateValue::Integer(320));

        Ok(())
    }

    #[test]
    fn destroying_socket_before_connection_prints_warning() -> Result<(), io::Error> {
        let (to_denon_client, dc) = create_connected_connection()?;
        {
            let _denon_client_destroy_socket = to_denon_client;
        }
        {
            let _dc_destroy = dc;
        }
        // TODO test still does not work, should print error
        Ok(())
    }

    #[test]
    fn read_without_valid_content_returns_empty_vec() -> Result<(), io::Error> {
        let listen_socket = TcpListener::bind("127.0.0.1:0")?;
        let addr = listen_socket.local_addr()?;
        let mut client = TcpStream::connect(addr)?;
        let (mut to_client, _) = listen_socket.accept()?;

        // as \r is missing, read() does not read or extract anything
        write_string(&mut to_client, "blub".to_string())?;
        let lines = read(&mut client, 1)?;
        assert_eq!(lines, Vec::<String>::new());

        // read() reads until \r and leaves other data in the stream
        write_string(&mut to_client, "bla\rfoo".to_string())?;
        let lines = read(&mut client, 2)?;
        assert_eq!(lines, vec!["blubbla".to_owned()]);

        Ok(())
    }
}
