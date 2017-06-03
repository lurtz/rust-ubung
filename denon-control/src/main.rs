// $ printf "MV53\r" | nc -i 1 0005cd221b08.lan 23 | stdbuf -o 0 tr "\r" "\n"
// MV53
// MVMAX 86

extern crate getopts;

mod denon_connection;
mod state;
mod operation;
mod parse;
mod pulseaudio;

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
    ops.optflag("l", "laptop", "move output to laptop");
    ops.optflag("r", "receiver", "move output to receiver and set volume");
    ops.optflag("s", "status", "print status of receiver");
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

    if args.opt_present("s") {
        print_status(&dc);
    }

    if args.opt_present("l") {
        pulseaudio::switch_ouput(pulseaudio::INTERNAL);
    }

    if args.opt_present("r") {
        if !args.opt_present("p") {
            dc.set(State::Power(PowerState::ON)).ok();
        }
        if !args.opt_present("i") {
            dc.set(State::SourceInput(SourceInputState::DVD)).ok();
        }
        if !args.opt_present("v") {
            dc.set(State::MainVolume(50)).ok();
        }
        pulseaudio::switch_ouput(pulseaudio::CUBIETRUCK);
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
}

