pub use state::State;
pub use state::PowerState;
pub use state::SourceInputState;
pub use operation::Operation;

macro_rules! parsehelper {
	($trimmed:expr, $op:path, $func:path) => {
		if $trimmed.starts_with($op.value()) {
          let value = get_value($trimmed, &$op);
  		  let x = $func(value, $op);
  		  return Some(x);
		}
	};
}

fn get_value<'a>(trimmed: &'a str, op: &Operation) -> &'a str {
    let to_skip = op.value().len();
    let ref to_parse = trimmed[to_skip..].trim();
    return to_parse;
}

fn parse_int(to_parse: &str) -> u32 {
    let mut value = to_parse.parse::<u32>().unwrap();
    if value < 100 {
        value = value * 10;
    }
    value
}

fn parse_main_volume(value: &str, op: Operation) -> (Operation, State) {
    let value = parse_int(value);
    return (op, State::MainVolume(value));
}

fn parse_max_volume(value: &str, op: Operation) -> (Operation, State) {
    let value = parse_int(value);
    return (op, State::MaxVolume(value));
}

fn parse_power(value: &str, op: Operation) -> (Operation, State) {
    if "ON" == value {
        return (op, State::Power(PowerState::ON));
    } else {
        return (op, State::Power(PowerState::STANDBY));
    }
}

fn parse_source_input(value: &str, op: Operation) -> (Operation, State) {
    if "DVD" == value {
        return (op, State::SourceInput(SourceInputState::DVD));
    } else if "GAME2" == value {
        return (op, State::SourceInput(SourceInputState::GAME2));
    }
    return (op, State::SourceInput(SourceInputState::NAPSTER));
}

pub fn parse(str: &str) -> Option<(Operation, State)> {
    let trimmed = str.trim().trim_matches('\r');
    parsehelper!(trimmed, Operation::MaxVolume, parse_max_volume);
    parsehelper!(trimmed, Operation::MainVolume, parse_main_volume);
    parsehelper!(trimmed, Operation::Power, parse_power);
    parsehelper!(trimmed, Operation::SourceInput, parse_source_input);
    None
}
