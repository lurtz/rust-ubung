pub use crate::operation::Operation;
pub use crate::state::PowerState;
pub use crate::state::SourceInputState;
pub use crate::state::State;
use crate::state::StateValue;

macro_rules! parsehelper {
    ($trimmed:expr, $op:expr, $func:path) => {
        if $trimmed.starts_with($op.value()) {
            let value = get_value($trimmed, &$op);
            let x = $func(value);
            return Some(($op, x));
        }
    };
}

fn get_value<'a>(trimmed: &'a str, op: &State) -> &'a str {
    let to_skip = op.value().len();
    trimmed[to_skip..].trim()
}

fn parse_int(to_parse: &str) -> u32 {
    let mut value = to_parse.parse::<u32>().unwrap();
    if value < 100 {
        value *= 10;
    }
    value
}

fn parse_main_volume(value: &str) -> StateValue {
    let value = parse_int(value);
    StateValue::Integer(value)
}

fn parse_max_volume(value: &str) -> StateValue {
    let value = parse_int(value);
    StateValue::Integer(value)
}

fn parse_power(value: &str) -> StateValue {
    if "ON" == value {
        StateValue::Power(PowerState::On)
    } else {
        StateValue::Power(PowerState::Standby)
    }
}

fn parse_source_input(value: &str) -> StateValue {
    for sis in SourceInputState::iterator() {
        if sis.to_string() == value {
            return StateValue::SourceInput(*sis);
        }
    }

    StateValue::SourceInput(SourceInputState::Unknown)
}

pub fn parse(str: &str) -> Option<(State, StateValue)> {
    let trimmed = str.trim().trim_matches('\r');
    parsehelper!(trimmed, State::MaxVolume, parse_max_volume);
    parsehelper!(trimmed, State::MainVolume, parse_main_volume);
    parsehelper!(trimmed, State::Power, parse_power);
    parsehelper!(trimmed, State::SourceInput, parse_source_input);
    None
}

#[cfg(test)]
mod test {
    use super::parse;
    use crate::{
        parse::{PowerState, SourceInputState, State},
        state::StateValue,
    };

    #[test]
    fn parse_with_unknown_string() {
        assert_eq!(None, parse(""));
        assert_eq!(None, parse("blub"));
    }

    #[test]
    #[should_panic]
    fn max_volume_without_value_panics() {
        parse("MVMAX");
    }

    #[test]
    fn max_volume() {
        let create = |i| Some((State::MaxVolume, StateValue::Integer(i)));

        assert_eq!(parse("MVMAX0"), create(0));
        assert_eq!(parse("MVMAX23"), create(230));
        assert_eq!(parse("MVMAX99"), create(990));
        assert_eq!(parse("MVMAX100"), create(100));
        assert_eq!(parse("MVMAX230"), create(230));
        assert_eq!(parse("MVMAX999"), create(999));
        assert_eq!(parse("MVMAX 999"), create(999));
    }

    #[test]
    #[should_panic]
    fn main_volume_without_value_panics() {
        parse("MV");
    }

    #[test]
    fn main_volume() {
        let create = |i| Some((State::MainVolume, StateValue::Integer(i)));

        assert_eq!(parse("MV 0"), create(0));
        assert_eq!(parse("MV 23"), create(230));
        assert_eq!(parse("MV 99"), create(990));
        assert_eq!(parse("MV 100"), create(100));
        assert_eq!(parse("MV 230"), create(230));
        assert_eq!(parse("MV 999"), create(999));
        assert_eq!(parse("MV999"), create(999));
    }

    #[test]
    fn power() {
        let create = |ps| Some((State::Power, StateValue::Power(ps)));

        assert_eq!(parse("PW"), create(PowerState::Standby));
        assert_eq!(parse("PWOFF"), create(PowerState::Standby));
        assert_eq!(parse("PWON"), create(PowerState::On));
    }

    #[test]
    fn source_input() {
        let create = |si| Some((State::SourceInput, StateValue::SourceInput(si)));

        assert_eq!(parse("SI"), create(SourceInputState::Unknown));
        assert_eq!(parse("SIblub"), create(SourceInputState::Unknown));
        assert_eq!(parse("SITV"), create(SourceInputState::Tv));
    }
}
