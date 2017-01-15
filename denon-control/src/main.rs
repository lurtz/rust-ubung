// $ printf "MV53\r" | nc -i 1 0005cd221b08.lan 23 | stdbuf -o 0 tr "\r" "\n"
// MV53
// MVMAX 86

use std::net::TcpStream;
use std::io::{Read, Write};
use std::time::Duration;
use std::thread;
use std::error::Error;
use std::ops::Index;
use std::string::ToString;
use std::fmt::Display;

#[cfg(test)]
mod test {
    #[test]
    fn bla() {
        assert_eq!(2, 2);
    }
}

fn do_it(denon_name: &str, denon_port: u16) -> Result<(), std::io::Error> {
    let mut stream = TcpStream::connect((denon_name, denon_port))?;

    println!("{}", stream.peer_addr()?);

    if process(&mut stream, &operations::power(), &"?")? {
        println!("power is on");
    } else {
        println!("power is off. turning on ...");
        process(&mut stream, &operations::power(), &"ON")?;
        thread::sleep(Duration::from_secs(1));
    }

    process(&mut stream, &operations::input(), &"?")?;

    // read volume first, commands which do not cause status changes will
    // not produce output
    let current_volume = process(&mut stream, &operations::volume(), &"?")?;
    process(&mut stream, &operations::volume(), &(current_volume / 2))?;

    thread::sleep(Duration::from_secs(1));

    process(&mut stream, &operations::volume(), &current_volume)?;

    Ok(())
}

fn write(stream: &mut Write, input: String) -> Result<(), std::io::Error> {
    println!("sending: {}", input);
    let volume_command = input.into_bytes();
    stream.write(&volume_command[..])?;
    Ok(())
}

fn read(stream: &mut Read, lines: u8) -> Result<Vec<String>, std::io::Error> {
    let mut string = String::new();

    for _ in 0..lines {
        let mut buffer = [0; 100];
        let read_bytes = stream.read(&mut buffer)?;

        if let Ok(tmp) = std::str::from_utf8(&buffer[0..read_bytes]) {
            string += tmp;
        }
    }

    string.pop();

    let string_iter = string.split('\r').map(|x| String::from(x));
    let result = string_iter.collect();
    println!("{:?}", result);
    Ok(result)
}

pub struct ControlElement<T> {
    prefix: &'static str,
    num_responses: u8,
    result_parser: Box<Fn(&str) -> T>,
}

impl<T> ControlElement<T> {
    fn new<F>(prefix: &'static str,
              num_responses: u8,
              result_parser: &'static F)
              -> ControlElement<T>
        where F: Fn(&str) -> T
    {
        ControlElement {
            prefix: prefix,
            num_responses: num_responses,
            result_parser: Box::new(result_parser),
        }
    }
}

mod operations {
    pub use ControlElement;
    use std::str::FromStr;

    pub fn volume() -> ControlElement<u32> {
        ControlElement {
            prefix: "MV",
            num_responses: 2,
            result_parser: Box::new(|x| u32::from_str(x).unwrap()),
        }
    }

    pub fn power() -> ControlElement<bool> {
        ControlElement {
            prefix: "PW",
            num_responses: 1,
            result_parser: Box::new(|x| if "ON" == x { true } else { false }),
        }
    }

    pub fn input() -> ControlElement<String> {
        ControlElement {
            prefix: "SI",
            num_responses: 2,
            result_parser: Box::new(|x| String::from(x)),
        }
    }
}

fn process<T, X>(stream: &mut T,
                 ce: &ControlElement<X>,
                 value: &Display)
                 -> Result<X, std::io::Error>
    where T: Write + Read
{
    let volume_command_string = format!("{}{}\r", ce.prefix, value.to_string());
    write(stream, volume_command_string)?;

    let result = read(stream, ce.num_responses)?;

    for volume_string in result {
        if volume_string.starts_with(ce.prefix) {
            let actual_value = volume_string.index(ce.prefix.len()..volume_string.len());
            let result = (ce.result_parser)(actual_value);
            return Ok(result);
        }
    }

    Err(std::io::Error::from(std::io::ErrorKind::InvalidData))
}

fn print_io_error(e: &std::io::Error) {
    println!("got error: {}, cause = {:?}, description = {}, kind = {:?}",
             e,
             e.cause(),
             e.description(),
             e.kind());
    if let Some(raw_os_error) = e.raw_os_error() {
        println!("raw_os_error = {}", raw_os_error);
    }
    if let Some(inner) = e.get_ref() {
        println!("inner = {}", inner);
    }
}

fn main() {
    let denon_name = "0005cd221b08.lan";
    let denon_port = 23;

    match do_it(denon_name, denon_port) {
        Ok(_) => println!("success"),
        Err(e) => {
            print_io_error(&e);
        }
    }
}
