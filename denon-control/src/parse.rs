pub use state::State;
pub use operation::Operation;

macro_rules! parsehelper {
	($trimmed:expr, $op:path, $func:path) => {
		let x = $func($trimmed, $op);
        if x.is_some() {
            return x;
        }
	};
}

fn parse_int(trimmed: &str, op: Operation) -> Option<(Operation, State)> {
    if trimmed.starts_with(op.value()) {
        let to_skip = op.value().len();
        let ref to_parse = trimmed[to_skip..].trim();
        let mut value = to_parse.parse::<u32>().unwrap();
        if value < 100 {
            value = value * 10;
        }
        return Some((op, State::Integer(value)));
    }
    None
}

fn parse_string(trimmed: &str, op: Operation) -> Option<(Operation, State)> {
    if trimmed.starts_with(op.value()) {
        let to_skip = op.value().len();
        let value = trimmed[to_skip..].to_string();
        return Some((op, State::String(value)));
    }
    None
}

pub fn parse(str: &str) -> Option<(Operation, State)> {
    let trimmed = str.trim().trim_matches('\r');
    parsehelper!(trimmed, Operation::MaxVolume, parse_int);
    parsehelper!(trimmed, Operation::MainVolume, parse_int);
    parsehelper!(trimmed, Operation::Power, parse_string);
    parsehelper!(trimmed, Operation::SourceInput, parse_string);
    None
}
