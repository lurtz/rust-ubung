#[derive(PartialEq, Eq, Debug)]
pub enum Operation {
    Query,
    Set,
    Stop,
}
