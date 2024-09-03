use std::io::{self, Read, Write};
use std::net::TcpStream;

#[cfg(test)]
use mockall::{automock, predicate::*};

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

#[cfg_attr(test, automock)]
pub trait ShutdownStream {
    fn shutdownly(&self) -> io::Result<()>;
    // TODO returning TcpStream breaks a bit the abstraction
    fn try_clonely(&self) -> io::Result<TcpStream>;
}

impl ShutdownStream for TcpStream {
    fn shutdownly(&self) -> io::Result<()> {
        self.shutdown(std::net::Shutdown::Both)
    }

    fn try_clonely(&self) -> io::Result<TcpStream> {
        self.try_clone()
    }
}

pub trait WriteShutdownStream: Write + ShutdownStream {}

impl WriteShutdownStream for TcpStream {}

pub fn create_tcp_stream(
    denon_name: String,
    denon_port: u16,
) -> Result<Box<dyn WriteShutdownStream>, io::Error> {
    let s = TcpStream::connect((denon_name.as_str(), denon_port))?;
    s.set_read_timeout(None)?;
    s.set_nonblocking(false)?;
    Ok(Box::new(s))
}

#[cfg(test)]
mod test {
    use std::{io, net::TcpListener};

    use crate::stream::create_tcp_stream;

    #[test]
    fn connects_to_server() -> Result<(), io::Error> {
        let listener = TcpListener::bind("localhost:0")?;
        let addr = listener.local_addr()?;
        assert!(create_tcp_stream(addr.ip().to_string(), addr.port()).is_ok());
        Ok(())
    }

    #[test]
    fn fails_to_connect_and_returns_unknown() {
        let dc = create_tcp_stream(String::from("value"), 0);
        assert!(matches!(dc, Err(_)));
    }
}
