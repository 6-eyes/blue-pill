use std::time::Duration;

const PORT: &str = "/dev/ttyUSB0";

fn main() -> ! {
    let mut port = serialport::new(PORT, 115200).timeout(Duration::from_secs(2)).open().expect("failed to open port");

    let mut buf = [0u8; 4];
    loop {
        port.read_exact(buf.as_mut_slice()).expect("unable to read data");
        let val: u32 = u32::from_le_bytes(buf);
        println!("received val: {val}");
    }
}
