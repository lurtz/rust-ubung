use std::process::Command;

pub fn get_receiver() -> String {
    let output = Command::new("/usr/bin/avahi-browse")
        .arg("-p")
        .arg("-t")
        .arg("-r")
        .arg("_raop._tcp")
        .output()
        .expect("avahi-browse failed");

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
        println!("No receiver found!");
        return String::new();
    } else {
        return String::from(denon_names[0]);
    }
}
