extern crate std;
use std::fmt::{Display, Formatter, Error};

#[derive(Debug,Clone)]
pub enum State {
    Integer(u32),
    String(String),
}

impl Display for State {
    fn fmt(&self, format: &mut Formatter) -> Result<(), Error> {
        match self {
            &State::Integer(i) => write!(format, "{}", i),
            &State::String(ref s) => write!(format, "{}", s),
        }
    }
}
