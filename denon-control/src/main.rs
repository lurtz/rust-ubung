// $ printf "MV53\r" | nc -i 1 0005cd221b08.lan 23 | stdbuf -o 0 tr "\r" "\n"
// MV53
// MVMAX 86

extern crate getopts;

mod denon_connection;
mod state;
mod operation;
mod parse;

use std::time::Duration;
use std::thread;

use denon_connection::{DenonConnection, State};
use state::PowerState;
use state::SourceInputState;

use getopts::Options;
use std::env;

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

fn parse_args() -> getopts::Matches {
    let mut ops = Options::new();
    ops.optopt("a", "address", "Address of Denon AVR", "HOSTNAME");
    ops.optopt("p", "power", "Power ON, STANDBY or OFF", "POWER_MODE");
    ops.optopt("v", "volume", "set volume in range 30..50", "VOLUME");
    ops.optopt("i", "input", "set source input: DVD, GAME2", "SOURCE_INPUT");
    ops.optflag("t", "test", "run old test code");
    ops.optflag("h", "help", "print help");

    let args : Vec<String> = env::args().collect();
    let arguments = match ops.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if arguments.opt_present("h") {
        let brief = format!("Usage: {} [options]", args[0]);
        print!("{}", ops.usage(&brief));
        std::process::exit(0);
    }

    arguments
}

fn print_status(dc : &DenonConnection) {
    println!("Current status of receiver:");
    println!("\t{:?}", dc.get(State::power()));
    println!("\t{:?}", dc.get(State::source_input()));
    println!("\t{:?}", dc.get(State::main_volume()));
    println!("\t{:?}", dc.get(State::max_volume()));
}

fn main_old(denon_name : &String, denon_port: u16) {
    let dc = DenonConnection::new(denon_name.as_str(), denon_port);

    let power_status = dc.get(State::power());
    println!("{:?}", power_status);
    if let Ok(State::Power(status)) = power_status {
        if status != PowerState::ON {
            dc.set(State::Power(PowerState::ON)).ok();
            thread::sleep(Duration::from_secs(1));
        }
    }
    println!("current input: {:?}", dc.get(State::source_input()));
    if let Ok(State::MainVolume(current_volume)) = dc.get(State::main_volume()) {
        dc.set(State::MainVolume(current_volume / 2)).ok();
        println!("{:?}", dc.get(State::main_volume()));
        thread::sleep(Duration::from_secs(5));
        dc.set(State::MainVolume(current_volume)).ok();
    }
    thread::sleep(Duration::from_secs(5));
    println!("{:?}", dc.get(State::main_volume()));
    println!("{:?}", dc.get(State::max_volume()));
    dc.stop().ok();
    thread::sleep(Duration::from_secs(5));
}

fn get_receiver_and_port(args : &getopts::Matches) -> (String, u16) {
    let mut denon_name = String::from("0005cd221b08.lan");
    if let Some(name) = args.opt_str("a") {
        denon_name = name;
    }
    (denon_name, 23)
}

fn main() {
    let args = parse_args();
    let (denon_name, denon_port) = get_receiver_and_port(&args);
    let dc = DenonConnection::new(denon_name.as_str(), denon_port);

    print_status(&dc);

    if args.opt_present("t") {
        main_old(&denon_name, denon_port);
        std::process::exit(0);
    }

    if let Some(p) = args.opt_str("p") {
        for power in PowerState::iterator() {
            if power.to_string() == p {
                dc.set(State::Power(power.clone())).ok();
            }
        }
    }

    if let Some(i) = args.opt_str("i") {
        for input in SourceInputState::iterator() {
            if input.to_string() == i {
                dc.set(State::SourceInput(input.clone())).ok();
            }
        }
    }

    if let Some(v) = args.opt_str("v") {
        let vi : u32 = v.parse().unwrap();
        dc.set(State::MainVolume(vi)).ok();
    }

    // need to check if thread in DenonConnection is stopped successfully to remove this
    // or need to check if each operation received a response
    thread::sleep(Duration::from_secs(1));
    print_status(&dc);
}

