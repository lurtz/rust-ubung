use crate::parse::parse;
pub use crate::parse::State;
use crate::state::{SetState, StateValue};

use std::collections::HashMap;
use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::panic;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[cfg(test)]
use mockall::{automock, predicate::*};

const ESHUTDOWN: i32 = 108;

#[cfg_attr(test, automock)]
pub trait ReadStream {
    fn peekly(&self, buf: &mut [u8]) -> io::Result<usize>;
    fn read_exactly(&self, buf: &mut [u8]) -> io::Result<()>;
}

impl ReadStream for TcpStream {
    fn peekly(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.peek(buf)
    }

    fn read_exactly(&self, buf: &mut [u8]) -> io::Result<()> {
        // why does this work?
        let mut mself = self;
        mself.read_exact(buf)
    }
}

fn write_string(stream: &mut dyn Write, input: String) -> Result<(), std::io::Error> {
    let volume_command = input.into_bytes();
    stream.write_all(&volume_command[..])?;
    Ok(())
}

pub fn write_state(stream: &mut dyn Write, state: SetState) -> Result<(), io::Error> {
    write_string(stream, format!("{}\r", state))
}

fn write_query(stream: &mut dyn Write, state: State) -> Result<(), io::Error> {
    write_string(stream, format!("{}?\r", state))
}

pub fn read(stream: &impl ReadStream, lines: u8) -> Result<Vec<String>, std::io::Error> {
    let mut result = Vec::<String>::new();

    // guarantee to read a full line. check that read content ends with \r
    while (lines as usize) != result.len() {
        let mut buffer = [0; 100];
        let read_bytes;
        match stream.peekly(&mut buffer) {
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

        stream.read_exactly(&mut buffer[0..bytes_to_extract])?;
    }

    Ok(result)
}

fn thread_func_impl(
    stream: &impl ReadStream,
    state: Arc<Mutex<HashMap<State, StateValue>>>,
) -> Result<(), std::io::Error> {
    loop {
        match read(stream, 1) {
            Ok(status_update) => {
                let parsed_response = parse_response(&status_update);
                let mut locked_state = state.lock().unwrap();
                for sstate in parsed_response {
                    let (state, value) = sstate.convert();
                    locked_state.insert(state, value);
                }
            }
            // check for timeout error -> continue on timeout error, else abort
            Err(e) => {
                if ErrorKind::TimedOut != e.kind() && ErrorKind::WouldBlock != e.kind() {
                    if e.raw_os_error() != Some(ESHUTDOWN) {
                        return Err(e);
                    }
                    return Ok(());
                }
            }
        }
    }
}

fn parse_response(response: &[String]) -> Vec<SetState> {
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
        s.set_read_timeout(None)?;
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
        write_query(&mut self.to_receiver, op)?;
        for _ in 0..50 {
            thread::sleep(Duration::from_millis(10));
            let locked_state = self.state.lock().unwrap();
            if let Some(state) = locked_state.get(&op) {
                return Ok(*state);
            }
        }
        Ok(StateValue::Unknown)
    }

    pub fn stop(&mut self) -> Result<(), io::Error> {
        self.to_receiver.shutdown(std::net::Shutdown::Both)
    }

    pub fn set(&mut self, sstate: SetState) -> Result<(), io::Error> {
        write_state(&mut self.to_receiver, sstate)
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
    use mockall::Sequence;

    use super::{thread_func_impl, DenonConnection, MockReadStream};
    use crate::denon_connection::{read, write_string};
    use crate::parse::{PowerState, SourceInputState};
    use crate::state::{SetState, State, StateValue};
    use std::cmp::min;
    use std::io::{self, Error};
    use std::net::{TcpListener, TcpStream};
    use std::sync::Arc;
    use std::thread::yield_now;

    pub fn create_connected_connection() -> Result<(TcpStream, DenonConnection), io::Error> {
        let listen_socket = TcpListener::bind("127.0.0.1:0")?;
        let addr = listen_socket.local_addr()?;
        let dc = DenonConnection::new(addr.ip().to_string(), addr.port())?;
        let (to_denon_client, _) = listen_socket.accept()?;
        Ok((to_denon_client, dc))
    }

    macro_rules! wait_for_value_in_database {
        ($denon_connection:ident, $sstate:expr) => {
            let (state, value) = $sstate.convert();
            for _ in 0..100000 {
                if $denon_connection.get(state)? == value {
                    break;
                }
                yield_now();
            }
        };
    }

    macro_rules! assert_db_value {
        ($denon_connection:ident, $sstate:expr) => {
            let (state, value) = $sstate.convert();
            assert_eq!($denon_connection.get(state)?, value);
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
        assert_db_value!(dc, SetState::MainVolume(234));
        Ok(())
    }

    #[test]
    fn connection_receives_multiple_values_volume_from_receiver() -> Result<(), io::Error> {
        let (mut to_denon_client, mut dc) = create_connected_connection()?;
        assert_eq!(StateValue::Unknown, dc.get(State::MainVolume)?);
        assert_eq!(StateValue::Unknown, dc.get(State::SourceInput)?);
        assert_eq!(StateValue::Unknown, dc.get(State::Power)?);
        write_string(&mut to_denon_client, "MV234\rSICD\rPWON\r".to_string())?;
        assert_db_value!(dc, SetState::MainVolume(234));
        assert_db_value!(dc, SetState::SourceInput(SourceInputState::Cd));
        assert_db_value!(dc, SetState::Power(PowerState::On));
        Ok(())
    }

    #[test]
    fn connection_updates_values_with_newly_received_data() -> Result<(), io::Error> {
        let (mut to_denon_client, mut dc) = create_connected_connection()?;
        write_string(&mut to_denon_client, "MV234\r".to_string())?;
        assert_db_value!(dc, SetState::MainVolume(234));
        write_string(&mut to_denon_client, "MV320\r".to_string())?;
        wait_for_value_in_database!(dc, SetState::MainVolume(320));
        assert_db_value!(dc, SetState::MainVolume(320));

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

    fn copy_string_into_slice(src: &str, dst: &mut [u8]) -> usize {
        let length = min(src.len(), dst.len());
        let mut chars = src.chars();
        for i in 0..length {
            dst[i] = chars.next().unwrap() as u8;
        }
        length
    }

    #[test]
    fn read_reads_content_gets_error_and_returns_content() -> Result<(), io::Error> {
        let mut sequence = Sequence::new();
        let mut mstream = MockReadStream::new();
        // peek works
        mstream
            .expect_peekly()
            .times(1)
            .in_sequence(&mut sequence)
            .returning(|buf| Ok(copy_string_into_slice("some_data\r", buf)));
        // read works
        mstream
            .expect_read_exactly()
            .times(1)
            .in_sequence(&mut sequence)
            .returning(|_| Ok(()));
        // peek with error
        mstream
            .expect_peekly()
            .times(1)
            .in_sequence(&mut sequence)
            .returning(|_| Err(Error::from(io::ErrorKind::ConnectionAborted)));
        let lines = read(&mut mstream, 2)?;
        assert_eq!(vec!(String::from("some_data")), lines);
        Ok(())
    }

    #[test]
    fn thread_func_impl_gets_error_and_returns() {
        let mut mstream = MockReadStream::new();
        mstream
            .expect_peekly()
            .returning(|_| Err(Error::from(io::ErrorKind::ConnectionAborted)));
        let state = Arc::default();
        let thread_err = thread_func_impl(&mstream, state);
        assert!(thread_err.is_err());
        assert_eq!(
            io::ErrorKind::ConnectionAborted,
            thread_err.unwrap_err().kind()
        );
    }

    #[test]
    fn thread_func_impl_gets_timeout_then_error_and_returns() {
        let mut sequence = Sequence::new();
        let mut mstream = MockReadStream::new();
        mstream
            .expect_peekly()
            .times(1)
            .in_sequence(&mut sequence)
            .returning(|_| Err(Error::from(io::ErrorKind::TimedOut)));
        mstream
            .expect_peekly()
            .times(1)
            .in_sequence(&mut sequence)
            .returning(|_| Err(Error::from(io::ErrorKind::ConnectionAborted)));
        let state = Arc::default();
        let thread_err = thread_func_impl(&mstream, state);
        assert!(thread_err.is_err());
        assert_eq!(
            io::ErrorKind::ConnectionAborted,
            thread_err.unwrap_err().kind()
        );
    }
}
