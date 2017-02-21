// $ printf "MV53\r" | nc -i 1 0005cd221b08.lan 23 | stdbuf -o 0 tr "\r" "\n"
// MV53
// MVMAX 86

mod denon_connection;
mod state;
mod operation;
mod parse;

use std::time::Duration;
use std::thread;

use denon_connection::{DenonConnection, State, Operation};

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

fn main() {
    let denon_name = "0005cd221b08.lan";
    let denon_port = 23;

    let dc = DenonConnection::new(denon_name, denon_port);
    let power_status = dc.get(Operation::Power);
    println!("{:?}", power_status);
    if let Ok(State::String(status)) = power_status {
        if status != "ON" {
            dc.set(Operation::Power, State::String(String::from("ON"))).ok();
            thread::sleep(Duration::from_secs(1));
        }
    }
    println!("current input: {:?}", dc.get(Operation::SourceInput));
    if let Ok(State::MainVolume(current_volume)) = dc.get(Operation::MainVolume) {
        dc.set(Operation::MainVolume, State::MainVolume(current_volume / 2)).ok();
        println!("{:?}", dc.get(Operation::MainVolume));
        thread::sleep(Duration::from_secs(5));
        dc.set(Operation::MainVolume, State::MainVolume(current_volume)).ok();
    }
    thread::sleep(Duration::from_secs(5));
    println!("{:?}", dc.get(Operation::MainVolume));
    println!("{:?}", dc.get(Operation::MaxVolume));
    dc.set(Operation::Stop, State::Integer(0)).ok();
    thread::sleep(Duration::from_secs(5));
}
