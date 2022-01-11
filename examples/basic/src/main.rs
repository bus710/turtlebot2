use std::thread;
use std::time::Duration;

use anyhow::Result;
use serialport::{SerialPortInfo, SerialPortType};

use turtlebot2::{rx, tx};

const SERIAL: &str = "kobuki";

fn main() {
    let ports = enum_ports().expect("Cannot enumerate ports");
    let mut found_kobuki = false;
    let mut found_kobuki_port_name = "";

    if ports.len() < 1 {
        panic!("No USB serial devices found")
    }

    for p in ports.iter() {
        match p.port_type.clone() {
            SerialPortType::UsbPort(info) => {
                if info.serial_number.unwrap().contains(SERIAL) {
                    eprintln!("Found port: {:?} - {:?}", p.port_name, p);
                    found_kobuki = true;
                    found_kobuki_port_name = &p.port_name;
                }
            }
            _ => found_kobuki = false,
        };
    }

    if found_kobuki {
        read_port(String::from(found_kobuki_port_name));
    }
}

fn enum_ports() -> Result<Vec<SerialPortInfo>> {
    let ports = serialport::available_ports().expect("No ports found!");
    Ok(ports)
}

fn read_port(port_name: String) {
    let mut port = serialport::new(port_name, 115200)
        .open()
        .expect("Open port");
    port.set_timeout(Duration::from_millis(1024))
        .expect("Setting timeout failed");

    let mut buffer = [0; 4096];
    let mut residue = Vec::new();

    for i in 0..3 {
        eprintln!("==================");
        eprintln!("{:?}", i);

        let len = port.read(&mut buffer).expect("Read failed");
        let d = rx::decode(&buffer[..len], &residue);
        match d {
            Ok(v) => {
                let (f, r) = v;
                eprintln!("f - {:?}", f);
                residue = r;
            }
            Err(_) => {} // Err(e) => {}
        }

        thread::sleep(Duration::from_millis(64)); // with 64 ms, the read returns about 220~350 bytes
    }

    eprintln!("==================");

    let p = tx::base_control_command(1, 1).expect(""); // subtle movement
    port.write(&p).expect("");

    thread::sleep(Duration::from_millis(100));

    let p = tx::base_control_command(0, 0).expect(""); // stop
    port.write(&p).expect("");
}
