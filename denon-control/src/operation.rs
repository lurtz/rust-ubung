extern crate std;

#[derive(PartialEq)]
pub enum Operation {
    Query,
    Set,
    Stop,
}
