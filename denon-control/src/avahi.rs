use std::process::Command;
use avahi_error::AvahiError;

#[allow(dead_code)]
pub fn get_receiver() -> Result<String, AvahiError> {
    let output = Command::new("/usr/bin/avahi-browse")
        .arg("-p")
        .arg("-t")
        .arg("-r")
        .arg("_raop._tcp")
        .output()?;

    let output_stdout = String::from_utf8_lossy(&output.stdout);
    let lines = output_stdout.lines();
    let denon_names : Vec<&str> = lines
        .filter(|&line| line.starts_with("="))
        .filter(|&line| line.contains("DENON"))
        .map(|line| line.split(';'))
        .map(|iter| iter.skip(6).next().unwrap())
        .collect();

    if denon_names.len() > 1 {
        println!("multiple receivers found: {:?}, taking: {}", denon_names, denon_names[0]);
        println!("use -a option if you want to use another receiver");
    }

    if denon_names.is_empty() {
        return Err(AvahiError::NoHostsFound);
    } else {
        return Ok(String::from(denon_names[0]));
    }
}
