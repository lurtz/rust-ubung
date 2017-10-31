use std::ffi::NulError;
use std::sync::{PoisonError, MutexGuard};
use std::convert::From;
use std::io;
use std::fmt;

pub enum AvahiError {
    PollerNew,
    ClientNew,
    CreateServiceBrowser(String, i32),
    NoHostsFound,
    MutexLocked,
    NulError(NulError),
    IOError(io::Error),
}

impl fmt::Display for AvahiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::AvahiError::*;

        write!(f, "AvahiError::")?;

        match self {
            &PollerNew => write!(f, "PollerNew"),
            &ClientNew => write!(f, "ClientNew"),
            &CreateServiceBrowser(ref message, ref rc) => write!(f, "CreateServiceBrowser: {}, avahi return code: {}", message, rc),
            &NoHostsFound => write!(f, "NoHostsFound"),
            &MutexLocked => write!(f, "MutexLocked"),
            &NulError(ref e) => write!(f, "NulError: {}", e),
            &IOError(ref e) => write!(f, "IOError: {}", e),
        }
    }
}

impl fmt::Debug for AvahiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

impl<'a, T> From<PoisonError<MutexGuard<'a, T>>> for AvahiError {
    fn from(_error: PoisonError<MutexGuard<'a, T>>) -> Self {
        AvahiError::MutexLocked
    }
}

impl From<NulError> for AvahiError {
    fn from(error: NulError) -> Self {
        AvahiError::NulError(error)
    }
}

impl From<io::Error> for AvahiError {
    fn from(error: io::Error) -> Self {
        AvahiError::IOError(error)
    }
}

