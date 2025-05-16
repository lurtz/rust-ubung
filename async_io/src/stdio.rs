#[cfg(test)]
use mockall::automock;
use std::io;
#[cfg(not(test))]
use std::io::Write;

#[cfg_attr(test, automock)]
// TODO make it clonable
pub trait Stdio {
    fn print(&self, line: &str) -> io::Result<usize>;
    fn flush(&self) -> io::Result<usize>;
    fn read_line(&self, buffer: &mut String) -> io::Result<usize>;
}

#[cfg(not(test))]
#[derive(Default)]
pub struct StdioImpl {}

#[cfg(not(test))]
impl Stdio for StdioImpl {
    fn print(&self, line: &str) -> io::Result<usize> {
        io::stdout().write(line.as_bytes())
    }

    fn flush(&self) -> io::Result<usize> {
        io::stdout().flush().map(|_| 0)
    }

    fn read_line(&self, buffer: &mut String) -> io::Result<usize> {
        io::stdin().read_line(buffer)
    }
}
