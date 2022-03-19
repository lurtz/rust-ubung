use std::convert::From;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io;

#[derive(Debug)]
pub enum Error {
    NoHostsFound,
    IO(io::Error),
    Zeroconf(zeroconf::error::Error),
}

impl Display for Error {
    fn fmt(&self, format: &mut Formatter) -> Result<(), fmt::Error> {
        write!(format, "{:?}", self)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IO(error)
    }
}

impl From<zeroconf::error::Error> for Error {
    fn from(error: zeroconf::error::Error) -> Self {
        Error::Zeroconf(error)
    }
}
