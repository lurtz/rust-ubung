use dbus;
use dbus::{BusType, Connection, Message};
use std::convert::From;
use std::error;
use std::fmt;

const EXTENSION_IFACE: &'static str = "org.gnome.Shell";
const EXTENSION_PATH: &'static str  = "/org/gnome/Shell";
const INTERFACE: &'static str = "org.gnome.Shell";

#[derive(Debug)]
pub enum Error {
    DBUS(dbus::Error),
    MethodCall(String),
    EVAL,
    JAVASCRIPT(String),
}

impl fmt::Display for Error {
    fn fmt(&self, format: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(format, "{:?}", self)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "Error for Gnome Shell operations"
    }
}

impl From<dbus::Error> for Error {
    fn from(err: dbus::Error) -> Self {
        Error::DBUS(err)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::MethodCall(err)
    }
}

pub struct GnomeShell {
    connection: Connection,
}

impl GnomeShell {
    pub fn new() -> Result<Self, Error> {
        Ok(GnomeShell{connection: Connection::get_private(BusType::Session)?})
    }

    pub fn eval(&self, js: &str) -> Result<String, Error> {
        let m = Message::new_method_call(EXTENSION_IFACE, EXTENSION_PATH, INTERFACE, "Eval")?.append1(js);
        let r = self.connection.send_with_reply_and_block(m, 2000)?;
        let (return_code, result) = r.get2::<bool, &str>();
        if return_code.is_none() || result.is_none() {
            return Err(Error::EVAL);
        }
        let string_result = String::from(result.unwrap());
        if return_code.unwrap() {
            Ok(String::from(string_result))
        } else {
            Err(Error::JAVASCRIPT(String::from(string_result)))
        }
    }
}

#[cfg(test)]
mod test {
    use gnome_shell::GnomeShell;

    #[test]
    fn gnome_shell() {
        let gs = GnomeShell::new().unwrap();
        let result = gs.eval("10+101").unwrap();
        assert!("111" == result);
    }
}

