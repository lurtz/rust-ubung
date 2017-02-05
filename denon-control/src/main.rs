// $ printf "MV53\r" | nc -i 1 0005cd221b08.lan 23 | stdbuf -o 0 tr "\r" "\n"
// MV53
// MVMAX 86

use std::net::TcpStream;
use std::io::{Read, Write};
use std::time::Duration;
use std::thread;
use std::ops::Index;
use std::fmt::Display;
use std::str::FromStr;

mod denon_connection;

use denon_connection::{DenonConnection, Operation, print_io_error, read, write};

#[cfg(test)]
mod test {
    #[test]
    fn bla() {
        assert_eq!(2, 2);
    }
}

// status object shall get the current status of the avr 1912
// easiest way would be a map<Key, Value> where Value is an enum of u32 and String
// Key is derived of a mapping from the protocol strings to constants -> define each string once
// the status object can be shared or the communication thread can be asked about a
// status which queries the receiver if it is not set

#[derive(Hash, Eq, PartialEq, Debug)]
enum Operations {
    MainVolume,
    Power,
    SourceInput,
}

impl Operations {
    fn value(&self) -> &'static str {
        match *self {
            Operations::MainVolume => "MV",
            Operations::Power => "PW",
            Operations::SourceInput => "SI",
        }
    }
}

fn do_it(denon_name: &str, denon_port: u16) -> Result<(), std::io::Error> {
    let mut stream = TcpStream::connect((denon_name, denon_port))?;

    println!("{}", stream.peer_addr()?);

    if process(&mut stream, &power(), &"?")? {
        println!("power is on");
    } else {
        println!("power is off. turning on ...");
        process(&mut stream, &power(), &"ON")?;
        thread::sleep(Duration::from_secs(1));
    }

    process(&mut stream, &input(), &"?")?;

    // read volume first, commands which do not cause status changes will
    // not produce output
    let current_volume = process(&mut stream, &volume(), &"?")?;
    process(&mut stream, &volume(), &(current_volume / 2))?;

    thread::sleep(Duration::from_secs(1));

    process(&mut stream, &volume(), &current_volume)?;

    Ok(())
}


// TODO add expected output
// TODO parse responses from denon asyncronously because they must not really
//      have to do to something with the command
pub struct ControlElement<T> {
    prefix: Operations,
    num_responses: u8,
    result_parser: Box<Fn(&str) -> T>,
}

impl<T> ControlElement<T> {
    fn new<F>(prefix: Operations, num_responses: u8, result_parser: F) -> ControlElement<T>
        where F: Fn(&str) -> T + 'static
    {
        ControlElement {
            prefix: prefix,
            num_responses: num_responses,
            result_parser: Box::new(result_parser),
        }
    }
}

pub fn volume() -> ControlElement<u32> {
    ControlElement::new(Operations::MainVolume, 2, |x| u32::from_str(x).unwrap())
}

pub fn power() -> ControlElement<bool> {
    ControlElement::new(Operations::Power,
                        1,
                        |x| if "ON" == x { true } else { false })
}

pub fn input() -> ControlElement<String> {
    ControlElement::new(Operations::SourceInput, 2, |x| String::from(x))
}

fn process<T, X>(stream: &mut T,
                 ce: &ControlElement<X>,
                 value: &Display)
                 -> Result<X, std::io::Error>
    where T: Write + Read
{
    let prefix = ce.prefix.value();
    let volume_command_string = format!("{}{}\r", prefix, value);
    write(stream, volume_command_string)?;

    let result = read(stream, ce.num_responses)?;

    for volume_string in result {
        if volume_string.starts_with(prefix) {
            let actual_value = volume_string.index(prefix.len()..volume_string.len());
            let result = (ce.result_parser)(actual_value);
            return Ok(result);
        }
    }

    Err(std::io::Error::from(std::io::ErrorKind::InvalidData))
}

fn main() {
    let denon_name = "0005cd221b08.lan";
    let denon_port = 23;

    //    match do_it(denon_name, denon_port) {
    //        Ok(_) => println!("success"),
    //        Err(e) => {
    //            print_io_error(&e);
    //        }
    //    }

    let dc = DenonConnection::new(denon_name, denon_port);
    println!("{:?}", dc.get(Operation::Power));
    println!("{:?}", dc.get(Operation::MainVolume));
    thread::sleep(Duration::from_secs(30));
}
