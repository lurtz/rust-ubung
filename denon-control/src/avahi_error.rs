use std::convert::From;
use std::error;
use std::ffi::NulError;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io;
use std::sync::{MutexGuard, PoisonError};
use std::time::SystemTimeError;

#[derive(Debug)]
pub enum Error {
    PollerNew,
    ClientNew,
    CreateServiceBrowser(String, i32),
    NoHostsFound,
    MutexLocked,
    Nul(NulError),
    IO(io::Error),
    SystemTime(SystemTimeError),
    Timeout,
}

impl Display for Error {
    fn fmt(&self, format: &mut Formatter) -> Result<(), fmt::Error> {
        write!(format, "{:?}", self)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "Error for Avahi related operations"
    }
}

impl<'a, T> From<PoisonError<MutexGuard<'a, T>>> for Error {
    fn from(_error: PoisonError<MutexGuard<'a, T>>) -> Self {
        Error::MutexLocked
    }
}

impl From<NulError> for Error {
    fn from(error: NulError) -> Self {
        Error::Nul(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IO(error)
    }
}

impl From<SystemTimeError> for Error {
    fn from(error: SystemTimeError) -> Self {
        Error::SystemTime(error)
    }
}
