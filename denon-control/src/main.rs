// $ printf "MV53\r" | nc -i 1 0005cd221b08.lan 23 | stdbuf -o 0 tr "\r" "\n"
// MV53
// MVMAX 86

use std::net::TcpStream;
use std::io::{Read, Write};
use std::time::Duration;
use std::thread;
use std::error::Error;
use std::ops::Index;
use std::str::FromStr;

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

    // read volume first, commands which do not cause status changes will
    // not produce output
    let current_volume = get_volume(&mut stream)?;
    set_volume(&mut stream, current_volume / 2)?;

    thread::sleep(Duration::from_secs(1));

    set_volume(&mut stream, current_volume)?;

    Ok(())
}

fn write(stream: &mut Write, input: String) -> Result<(), std::io::Error> {
    println!("sending: {}", input);
    let volume_command = input.into_bytes();
    stream.write(&volume_command[..])?;
    Ok(())
}

fn read(stream: &mut Read, lines: Option<u8>) -> Result<Vec<String>, std::io::Error> {
    let mut string = String::new();
    let limit = lines.unwrap_or(1);

    for _ in 0..limit {
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

fn get_volume<T>(stream: &mut T) -> Result<u32, std::io::Error>
    where T: Write + Read
{
    let volume_command_string = String::from("MV?\r");
    write(stream, volume_command_string)?;

    let result = read(stream, Some(2))?;
    let ref volume_string = result[0];
    let actual_value = volume_string.index(2..volume_string.len());

    let result = u32::from_str(actual_value).unwrap();

    Ok(result)
}

fn set_volume<T>(stream: &mut T, volume: u32) -> Result<Vec<String>, std::io::Error>
    where T: Read + Write
{
    let volume_command_string = format!("MV{}\r", volume);
    write(stream, volume_command_string)?;
    read(stream, Some(2))
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
