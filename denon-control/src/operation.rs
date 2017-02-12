extern crate std;
use std::fmt::{Display, Formatter};

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Operation {
    MaxVolume,
    MainVolume,
    Power,
    SourceInput,
    Stop,
}

impl Operation {
    pub fn value(&self) -> &'static str {
        match *self {
            Operation::MaxVolume => "MVMAX",
            Operation::MainVolume => "MV",
            Operation::Power => "PW",
            Operation::SourceInput => "SI",
            Operation::Stop => "really stop now",
        }
    }
}

impl Display for Operation {
    fn fmt(&self, format: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(format, "{}", self.value())
    }
}
