// $ printf "MV53\r" | nc -i 1 0005cd221b08.lan 23 | stdbuf -o 0 tr "\r" "\n"
// MV53
// MVMAX 86

mod avahi;
mod avahi3;
mod avahi_error;
mod denon_connection;
mod parse;
mod state;

use denon_connection::{DenonConnection, State};
use state::{PowerState, SetState, SourceInputState};

use getopts::Options;
use std::env;
use std::fmt;

// status object shall get the current status of the avr 1912
// easiest way would be a map<Key, Value> where Value is an enum of u32 and String
// Key is derived of a mapping from the protocol strings to constants -> define each string once
// the status object can be shared or the communication thread can be asked about a
// status which queries the receiver if it is not set

fn parse_args(args: Vec<String>) -> getopts::Matches {
    let mut ops = Options::new();
    ops.optopt("a", "address", "Address of Denon AVR", "HOSTNAME");
    ops.optopt("p", "power", "Power ON, STANDBY or OFF", "POWER_MODE");
    ops.optopt("v", "volume", "set volume in range 30..50", "VOLUME");
    ops.optopt("i", "input", "set source input: DVD, GAME2", "SOURCE_INPUT");
    ops.optflag(
        "e",
        "extern-avahi",
        "use avahi-browser to find receiver instead of library",
    );
    ops.optflag("s", "status", "print status of receiver");
    ops.optflag("h", "help", "print help");

    let arguments = match ops.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f.to_string())
        }
    };

    if arguments.opt_present("h") {
        let brief = format!("Usage: {} [options]", args[0]);
        print!("{}", ops.usage(&brief));
        let exit_success: i32 = 0;
        std::process::exit(exit_success);
    }

    arguments
}

fn print_status(dc: &mut DenonConnection) -> Result<String, std::io::Error> {
    Ok(format!(
        "Current status of receiver:\n\tPower({})\n\tSourceInput({})\n\tMainVolume({})\n\tMaxVolume({})\n",
        dc.get(State::Power)?,
        dc.get(State::SourceInput)?,
        dc.get(State::MainVolume)?,
        dc.get(State::MaxVolume)?
    ))
}

fn get_avahi_impl(args: &getopts::Matches) -> fn() -> Result<String, avahi_error::Error> {
    if args.opt_present("e") {
        avahi::get_receiver
    } else {
        avahi3::get_receiver
    }
}

fn get_receiver_and_port(
    args: &getopts::Matches,
    get_rec: fn() -> Result<String, avahi_error::Error>,
) -> Result<(String, u16), avahi_error::Error> {
    let denon_name = match args.opt_str("a") {
        Some(name) => name,
        None => get_rec()?,
    };
    println!("using receiver: {}", denon_name);
    Ok((denon_name, 23))
}

#[derive(Debug)]
#[allow(dead_code)] // Fields will be used when an error is printed
enum Error {
    ParseInt(std::num::ParseIntError),
    Avahi(avahi_error::Error),
    IO(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, format: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(format, "{:?}", self)
    }
}

impl std::convert::From<std::num::ParseIntError> for Error {
    fn from(parse_error: std::num::ParseIntError) -> Self {
        Error::ParseInt(parse_error)
    }
}

impl std::convert::From<avahi_error::Error> for Error {
    fn from(avahi_error: avahi_error::Error) -> Self {
        Error::Avahi(avahi_error)
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(io_error: std::io::Error) -> Self {
        Error::IO(io_error)
    }
}

fn main2(args: getopts::Matches, denon_name: String, denon_port: u16) -> Result<(), Error> {
    let mut dc = DenonConnection::new(denon_name, denon_port)?;

    if args.opt_present("s") {
        println!("{}", print_status(&mut dc)?);
    }

    if let Some(p) = args.opt_str("p") {
        for power in PowerState::iterator() {
            if power.to_string() == p {
                dc.set(SetState::Power(*power))?;
            }
        }
    }

    if let Some(i) = args.opt_str("i") {
        for input in SourceInputState::iterator() {
            if input.to_string() == i {
                dc.set(SetState::SourceInput(*input))?;
            }
        }
    }

    if let Some(v) = args.opt_str("v") {
        let mut vi: u32 = v.parse()?;
        // do not accidentally kill the ears
        if vi > 50 {
            vi = 50;
        }
        dc.set(SetState::MainVolume(vi))?;
    }
    Ok(())
}

fn main() -> Result<(), Error> {
    let args = parse_args(env::args().collect());
    let (denon_name, denon_port) = get_receiver_and_port(&args, get_avahi_impl(&args))?;
    main2(args, denon_name, denon_port)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::avahi;
    use crate::avahi3;
    use crate::avahi_error;
    use crate::denon_connection::read;
    use crate::denon_connection::write_state;
    use crate::get_avahi_impl;
    use crate::get_receiver_and_port;
    use crate::main2;
    use crate::state::SetState;
    use crate::Error;
    use crate::PowerState;
    use crate::SourceInputState;
    use crate::{
        denon_connection::test::create_connected_connection, parse::State, parse_args, print_status,
    };
    use std::io;
    use std::net::TcpListener;
    use std::thread;

    #[test]
    #[should_panic]
    fn parse_args_parnics_with_empty_vec() {
        parse_args(vec![]);
    }

    #[test]
    #[should_panic]
    fn parse_args_parnics_with_unknown_option() {
        let string_args = vec!["blub", "-w"];
        parse_args(string_args.into_iter().map(|a| a.to_string()).collect());
    }

    #[test]
    fn parse_args_works_with_empty_strings() {
        parse_args(vec!["".to_string()]);
        parse_args(vec!["blub".to_string()]);
    }

    #[test]
    fn parse_args_short_options() {
        let string_args = vec![
            "blub",
            "-a",
            "some_host",
            "-p",
            "OFF",
            "-v",
            "20",
            "-i",
            "DVD",
            "-e",
            "-s",
        ];
        let args = parse_args(string_args.into_iter().map(|a| a.to_string()).collect());
        assert!(matches!(args.opt_str("a"), Some(x) if x == "some_host"));
        assert!(matches!(args.opt_str("p"), Some(x) if x == "OFF"));
        assert!(matches!(args.opt_str("v"), Some(x) if x == "20"));
        assert!(matches!(args.opt_get::<u32>("v"), Ok(Some(x)) if x == 20));
        assert!(matches!(args.opt_str("i"), Some(x) if x == "DVD"));
        assert!(args.opt_present("e"));
        assert!(args.opt_present("s"));
    }

    #[test]
    fn parse_args_long_options() {
        let string_args = vec![
            "blub",
            "--address",
            "some_host",
            "--power",
            "OFF",
            "--volume",
            "20",
            "--input",
            "DVD",
            "--extern-avahi",
            "--status",
        ];
        let args = parse_args(string_args.into_iter().map(|a| a.to_string()).collect());
        assert!(matches!(args.opt_str("a"), Some(x) if x == "some_host"));
        assert!(matches!(args.opt_str("p"), Some(x) if x == "OFF"));
        assert!(matches!(args.opt_str("v"), Some(x) if x == "20"));
        assert!(matches!(args.opt_get::<u32>("v"), Ok(Some(x)) if x == 20));
        assert!(matches!(args.opt_str("i"), Some(x) if x == "DVD"));
        assert!(args.opt_present("e"));
        assert!(args.opt_present("s"));
    }

    #[test]
    fn print_status_test() -> Result<(), io::Error> {
        let (mut to_receiver, mut dc) = create_connected_connection()?;
        write_state(&mut to_receiver, SetState::Power(PowerState::On))?;
        write_state(
            &mut to_receiver,
            SetState::SourceInput(SourceInputState::Cd),
        )?;
        write_state(&mut to_receiver, SetState::MainVolume(230))?;
        write_state(&mut to_receiver, SetState::MaxVolume(666))?;

        let expected = "Current status of receiver:\n\tPower(ON)\n\tSourceInput(CD)\n\tMainVolume(230)\n\tMaxVolume(666)\n";
        assert_eq!(expected, print_status(&mut dc).unwrap());
        Ok(())
    }

    #[test]
    fn get_avahi_impl_extern_test() {
        let string_args = vec!["blub", "--extern-avahi"];
        let args = parse_args(string_args.into_iter().map(|a| a.to_string()).collect());

        assert_eq!(
            avahi::get_receiver as fn() -> Result<String, crate::avahi_error::Error>,
            get_avahi_impl(&args)
        );
    }

    #[test]
    fn get_avahi_impl_intern_test() {
        let string_args = vec!["blub"];
        let args = parse_args(string_args.into_iter().map(|a| a.to_string()).collect());

        assert_eq!(
            avahi3::get_receiver as fn() -> Result<String, crate::avahi_error::Error>,
            get_avahi_impl(&args)
        );
    }

    #[test]
    fn get_receiver_and_port_using_avahi_test() -> Result<(), Error> {
        let string_args = vec!["blub"];
        let args = parse_args(string_args.into_iter().map(|a| a.to_string()).collect());
        let receiver_address = String::from("some_receiver");
        assert_eq!(
            (receiver_address, 23),
            get_receiver_and_port(&args, || Ok(String::from("some_receiver")))?
        );
        Ok(())
    }

    #[test]
    fn get_receiver_and_port_using_avahi_fails_test() -> Result<(), Error> {
        let string_args = vec!["blub"];
        let args = parse_args(string_args.into_iter().map(|a| a.to_string()).collect());
        assert!(matches!(
            get_receiver_and_port(&args, || Err(avahi_error::Error::NoHostsFound)),
            Err(avahi_error::Error::NoHostsFound)
        ));
        Ok(())
    }

    #[test]
    fn get_receiver_and_port_using_args_test() -> Result<(), Error> {
        let string_args = vec!["blub", "-a", "blub_receiver"];
        let args = parse_args(string_args.into_iter().map(|a| a.to_string()).collect());
        let receiver_address = String::from("blub_receiver");
        assert_eq!(
            (receiver_address, 23),
            get_receiver_and_port(&args, || Ok(String::from("some_receiver")))?
        );
        Ok(())
    }

    #[test]
    fn main2_test() -> Result<(), io::Error> {
        let listen_socket = TcpListener::bind("127.0.0.1:0")?;
        let local_port = listen_socket.local_addr()?.port();
        let string_args = vec![
            "blub",
            "-a",
            "localhost",
            "-s",
            "-p",
            "STANDBY",
            "-i",
            "CD",
            "-v",
            "127",
        ];
        let args = parse_args(string_args.into_iter().map(|a| a.to_string()).collect());

        let acceptor = thread::spawn(move || -> Result<Vec<String>, io::Error> {
            let mut to_receiver = listen_socket.accept()?.0;

            write_state(&mut to_receiver, SetState::Power(PowerState::On))?;
            write_state(
                &mut to_receiver,
                SetState::SourceInput(SourceInputState::Dvd),
            )?;
            write_state(&mut to_receiver, SetState::MainVolume(230))?;
            write_state(&mut to_receiver, SetState::MaxVolume(666))?;
            // might contain status queries
            read(&mut to_receiver, 10)
        });

        main2(args, String::from("localhost"), local_port).unwrap();

        let received_data = acceptor.join().unwrap()?;
        assert!(received_data.contains(&format!("{}?", State::Power)));
        assert!(received_data.contains(&format!("{}", SetState::SourceInput(SourceInputState::Cd))));
        assert!(received_data.contains(&format!("{}", SetState::MainVolume(50))));
        assert!(received_data.contains(&format!("{}", SetState::Power(PowerState::Standby))));
        Ok(())
    }

    macro_rules! check_error {
        ($error_value:expr, $expected:pat, $string:expr ) => {
            let error = Error::from($error_value);
            assert!(matches!(error, $expected));
            assert_eq!($string, format!("{}", error));
        };
    }

    #[test]
    fn error_test() {
        check_error!(
            i32::from_str_radix("a23", 10).unwrap_err(),
            Error::ParseInt(_),
            "ParseInt(ParseIntError { kind: InvalidDigit })"
        );
        check_error!(
            avahi_error::Error::NoHostsFound,
            Error::Avahi(_),
            "Avahi(NoHostsFound)"
        );
        check_error!(
            std::io::Error::from(io::ErrorKind::AddrInUse),
            Error::IO(_),
            "IO(Kind(AddrInUse))"
        );
    }
}
