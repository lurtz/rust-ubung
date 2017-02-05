// $ printf "MV53\r" | nc -i 1 0005cd221b08.lan 23 | stdbuf -o 0 tr "\r" "\n"
// MV53
// MVMAX 86

use std::time::Duration;
use std::thread;

mod denon_connection;

use denon_connection::{DenonConnection, Operation, State};

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
    println!("{:?}", dc.get(Operation::Power));
    if let State::Integer(current_volume) = dc.get(Operation::MainVolume) {
        dc.set(Operation::MainVolume, State::Integer(current_volume / 2));
        println!("{:?}", dc.get(Operation::MainVolume));
        thread::sleep(Duration::from_secs(5));
        dc.set(Operation::MainVolume, State::Integer(current_volume));
    }
    thread::sleep(Duration::from_secs(5));
    println!("{:?}", dc.get(Operation::MainVolume));
    println!("{:?}", dc.get(Operation::MaxVolume));
    thread::sleep(Duration::from_secs(20));
}
