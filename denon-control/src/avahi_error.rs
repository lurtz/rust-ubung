use std::ffi::NulError;
use std::sync::{PoisonError, MutexGuard};
use std::convert::From;
use std::io;

pub enum AvahiError {
    PollerNew,
    ClientNew,
    CreateServiceBrowser(String, i32),
    NoHostsFound,
    MutexLocked,
    NulError(NulError),
    IOError(io::Error),
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

