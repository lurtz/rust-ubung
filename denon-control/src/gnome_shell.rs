use dbus;
use dbus::{BusType, Connection, Message};
use std::convert::From;

const EXTENSION_IFACE: &'static str = "org.gnome.Shell";
const EXTENSION_PATH: &'static str  = "/org/gnome/Shell";
const INTERFACE: &'static str = "org.gnome.Shell";

#[derive(Debug)]
enum Error {
    DBUS(dbus::Error),
    MethodCall(String),
    EVAL,
    JAVASCRIPT(String),
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

struct GnomeShell {
    connection: Connection,
}

impl GnomeShell {
    fn new() -> Result<Self, Error> {
        Ok(GnomeShell{connection: Connection::get_private(BusType::Session)?})
    }

    fn eval(&self, js: &str) -> Result<String, Error> {
        let m = Message::new_method_call(EXTENSION_IFACE, EXTENSION_PATH, INTERFACE, "Eval")?.append1(js);
        let r = self.connection.send_with_reply_and_block(m, 2000)?;
        let (return_code, result) = r.get2::<bool, &str>();
        if return_code.is_none() || result.is_none() {
            return Err(Error::EVAL);
        }
        if !return_code.unwrap() {
            Err(Error::JAVASCRIPT(String::from(result.unwrap())))
        } else {
            Ok(String::from(result.unwrap()))
        }
    }
}

#[cfg(test)]
mod test {
    use dbus::*;
    use gnome_shell::*;

    #[test]
    fn gnome_shell() {
        let gs = GnomeShell::new().unwrap();
        let result = gs.eval("10+101").unwrap();
        assert!("111" == result);
    }

    #[test]
    fn dbus_test() {
        let c = Connection::get_private(BusType::Session).unwrap();
        let m = Message::new_method_call(EXTENSION_IFACE, EXTENSION_PATH, INTERFACE, "Eval").unwrap().append1("10+101");
        let r = c.send_with_reply_and_block(m, 2000).unwrap();
        let (return_code, result) = r.get2::<bool, &str>();
        println!("return_code: {:?}, result: {:?}", return_code, result);
        assert!(true == return_code.unwrap());
        assert!("111" == result.unwrap());
    }
}

