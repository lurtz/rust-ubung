pub use crate::operation::Operation;
pub use crate::state::PowerState;
pub use crate::state::SourceInputState;
pub use crate::state::State;

macro_rules! parsehelper {
    ($trimmed:expr, $op:expr, $func:path) => {
        if $trimmed.starts_with($op.value()) {
            let value = get_value($trimmed, &$op);
            let x = $func(value);
            return Some(x);
        }
    };
}

fn get_value<'a>(trimmed: &'a str, op: &State) -> &'a str {
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

fn parse_main_volume(value: &str) -> State {
    let value = parse_int(value);
    return State::MainVolume(value);
}

fn parse_max_volume(value: &str) -> State {
    let value = parse_int(value);
    return State::MaxVolume(value);
}

fn parse_power(value: &str) -> State {
    if "ON" == value {
        return State::Power(PowerState::ON);
    } else {
        return State::Power(PowerState::STANDBY);
    }
}

fn parse_source_input(value: &str) -> State {
    for sis in SourceInputState::iterator() {
        if sis.to_string() == value {
            return State::SourceInput(sis.clone());
        }
    }

    return State::SourceInput(SourceInputState::UNKNOWN);
}

pub fn parse(str: &str) -> Option<State> {
    let trimmed = str.trim().trim_matches('\r');
    parsehelper!(trimmed, State::max_volume(), parse_max_volume);
    parsehelper!(trimmed, State::main_volume(), parse_main_volume);
    parsehelper!(trimmed, State::power(), parse_power);
    parsehelper!(trimmed, State::source_input(), parse_source_input);
    None
}
