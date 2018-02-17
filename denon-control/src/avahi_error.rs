use std::ffi::NulError;
use std::sync::{PoisonError, MutexGuard};
use std::convert::From;
use std::io;
use std::time::SystemTimeError;
use std::error;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    PollerNew,
    ClientNew,
    CreateServiceBrowser(String, i32),
    NoHostsFound,
    MutexLocked,
    NulError(NulError),
    IOError(io::Error),
    SystemTimeError(SystemTimeError),
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
        Error::NulError(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IOError(error)
    }
}

impl From<SystemTimeError> for Error {
    fn from(error: SystemTimeError) -> Self {
        Error::SystemTimeError(error)
    }
}

