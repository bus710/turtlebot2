use std::thread;
use std::time::Duration;

use serialport::SerialPortType;
use turtlebot2::rx;

const SERIAL: &str = "kobuki";

fn main() {
    // Need to check if there are ports available
    let ports = serialport::available_ports().expect("No ports found!");
    if ports.len() < 1 {
        panic!("No USB serial devices found")
    }

    // Need to check if there is any port that has serial number with the given string "kobuki"
    let mut found_kobuki = false;
    let mut found_kobuki_port_name = "";
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

    // If there is, read the port and parse the byte stream
    if found_kobuki {
        test_port(String::from(found_kobuki_port_name));
    }
}

fn test_port(port_name: String) {
    let mut port = serialport::new(port_name, 115200)
        .open()
        .expect("Failed to open the port");
    port.set_timeout(Duration::from_millis(1024))
        .expect("Failed to set the timeout");

    let mut buffer = [0; 4096]; // To read bytes from port
    let mut residue = Vec::new(); // To keep broken packets between iteration
    for i in 0..3 {
        eprintln!("==================");
        eprintln!("Iteration - {:?}", i);

        let len = port.read(&mut buffer).expect("Read failed");
        let d = rx::decode(&buffer[..len], &residue);
        match d {
            Ok(v) => {
                let (f, r) = v;
                eprintln!("Number of feedbacks found - {:?}", f.len());
                residue = r;
            }
            Err(_) => {
                eprintln!("Found a broken packet only")
            } // Err(e) => {}
        }

        thread::sleep(Duration::from_millis(64)); // with 64 ms, the read returns about 220~350 bytes
    }

    eprintln!("==================");

    // let p = tx::base_control_command(1, 1).expect(""); // subtle movement
    // port.write(&p).expect("");
    // thread::sleep(Duration::from_millis(100));
    // let p = tx::base_control_command(0, 0).expect(""); // stop
    // port.write(&p).expect("");
}
