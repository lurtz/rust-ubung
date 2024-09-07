use assert_cmd::prelude::*; // Add methods on commands
use denon_control::{read, write_string};
use predicates::prelude::*; // Used for writing assertions
use predicates::str::contains;
use std::{
    io,
    net::{TcpListener, TcpStream},
    process::Command,
    thread,
}; // Run programs

#[test]
fn denon_control_prints_help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("denon-control")?;
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage"));

    Ok(())
}

#[test]
fn denon_control_fails_to_connect() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("denon-control")?;
    cmd.arg("--address").arg("localhost");
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("using receiver: localhost:"))
        .stderr(predicate::str::contains("Connection refused"));

    Ok(())
}

#[test]
fn denon_control_connects_to_test_receiver() -> Result<(), Box<dyn std::error::Error>> {
    let listen_socket = TcpListener::bind("localhost:0")?;
    let local_port = listen_socket.local_addr()?.port();

    let mut cmd = Command::cargo_bin("denon-control")?;
    cmd.arg("--address")
        .arg(format!("localhost:{}", local_port));
    cmd.assert()
        .success()
        .stdout(contains("using receiver: localhost:"));

    Ok(())
}

#[test]
fn denon_control_queries_receiver_state() -> Result<(), Box<dyn std::error::Error>> {
    let listen_socket = TcpListener::bind("localhost:0")?;
    let local_port = listen_socket.local_addr()?.port();
    let mut cmd = Command::cargo_bin("denon-control")?;

    let acceptor = thread::spawn(move || -> Result<(TcpStream, Vec<String>), io::Error> {
        let mut to_receiver = listen_socket.accept()?.0;
        let mut received_data = read(&mut to_receiver, 1)?;
        write_string(&mut to_receiver, String::from("PWON\r"))?;
        received_data.append(&mut read(&mut to_receiver, 1)?);
        write_string(&mut to_receiver, String::from("SIDVD\r"))?;
        received_data.append(&mut read(&mut to_receiver, 1)?);
        write_string(&mut to_receiver, String::from("MV230\r"))?;
        received_data.append(&mut read(&mut to_receiver, 1)?);
        write_string(&mut to_receiver, String::from("MVMAX666\r"))?;
        Ok((to_receiver, received_data))
    });

    let expected = "Current status of receiver:\n\tPower(ON)\n\tSourceInput(DVD)\n\tMainVolume(230)\n\tMaxVolume(666)\n";

    cmd.arg("--address")
        .arg(format!("localhost:{}", local_port))
        .arg("--status");
    cmd.assert().success().stdout(contains(expected));

    let (_, received_data) = acceptor.join().unwrap()?;

    assert!(received_data.contains(&String::from("PW?")));
    assert!(received_data.contains(&String::from("SI?")));
    assert!(received_data.contains(&String::from("MV?")));
    assert!(received_data.contains(&String::from("MVMAX?")));

    Ok(())
}

#[test]
fn denon_control_sets_receiver_state() -> Result<(), Box<dyn std::error::Error>> {
    let listen_socket = TcpListener::bind("localhost:0")?;
    let local_port = listen_socket.local_addr()?.port();
    let mut cmd = Command::cargo_bin("denon-control")?;

    let acceptor = thread::spawn(move || -> Result<TcpStream, io::Error> {
        let to_receiver = listen_socket.accept()?.0;
        Ok(to_receiver)
    });

    cmd.arg("--address")
        .arg(format!("localhost:{}", local_port))
        .arg("--power")
        .arg("STANDBY")
        .arg("--input")
        .arg("CD")
        .arg("--volume")
        .arg("127");
    cmd.assert()
        .success()
        .stdout(contains("using receiver: localhost:"));

    let mut to_receiver = acceptor.join().unwrap()?;
    let received_data = read(&mut to_receiver, 10)?;

    assert!(received_data.contains(&String::from("SICD")));
    assert!(received_data.contains(&String::from("MV50")));
    assert!(received_data.contains(&String::from("PWSTANDBY")));

    Ok(())
}
