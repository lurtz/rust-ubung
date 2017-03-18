extern crate std;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Operation {
    Query,
    Set,
    Stop,
}
