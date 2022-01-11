use std::thread;
use std::time::Duration;

use anyhow::Result;
use serialport::SerialPortInfo;

use turtlebot2::{rx, tx};

const PORT: &str = "/dev/ttyUSB0";

fn main() {
    let ports = enum_ports().expect("cannot enumerate ports");
    let mut found_ttyusb0 = false;
    if ports.len() < 1 {
        panic!("No USB serial devices found")
    }

    for p in ports.iter() {
        eprintln!("{:?} - {:?}", p.port_name, p);
        if p.port_name.contains(PORT) {
            found_ttyusb0 = true;
        }
    }

    if found_ttyusb0 {
        read_port(String::from(PORT));
    }
}

fn enum_ports() -> Result<Vec<SerialPortInfo>> {
    let ports = serialport::available_ports().expect("No ports found!");
    Ok(ports)
}

fn read_port(port_name: String) {
    eprintln!("{:?}", port_name);

    let mut port = serialport::new(port_name, 115200)
        .open()
        .expect("Open port");
    port.set_timeout(Duration::from_millis(1024))
        .expect("Setting timeout failed");

    let mut buffer = [0; 4096];
    let mut residue = Vec::new();

    for i in 0..10 {
        let len = port.read(&mut buffer).expect("Read failed");
        let d = rx::decode(&buffer[..len], &residue);
        match d {
            Ok(v) => {
                let (f, r) = v;
                eprintln!("f - {:?}", f);
                residue = r;
            }
            Err(e) => {
                eprintln!("Error - {:?}", e);
            }
        }
        eprintln!("================== {:?}", i);
        thread::sleep(Duration::from_millis(64)); // with 64 ms, the read returns about 220~350 bytes
    }

    let p = tx::base_control_command(1, 1).expect(""); // subtle movement
    port.write(&p).expect("");

    thread::sleep(Duration::from_millis(100));

    let p = tx::base_control_command(0, 0).expect(""); // stop
    port.write(&p).expect("");
}
