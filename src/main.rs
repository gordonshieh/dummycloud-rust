extern crate bytes;
extern crate crypto;

use std::net::UdpSocket;
use std::str;
use std::ascii::escape_default;
use std::time::SystemTime;

use bytes::{BytesMut, Buf, BufMut};

mod codec;

fn show(bs: BytesMut) -> String {
    let mut visible = String::new();
    for &b in bs.bytes() {
        let part: Vec<u8> = escape_default(b).collect();
        visible.push_str(str::from_utf8(&part).unwrap());
    }
    visible
}

fn show_bytes(bs: &[u8]) -> String {
    let mut visible = String::new();
    for &b in bs {
        let part: Vec<u8> = escape_default(b).collect();
        visible.push_str(str::from_utf8(&part).unwrap());
    }
    visible
}

fn create_timesync_packet() -> BytesMut {
    let mut packet = BytesMut::with_capacity(32);

    // set headers for sending timesync packet
    packet.put_u8(0x21);
    packet.put_u8(0x31);
    packet.put_u8(0x00);
    packet.put_u8(0x20);
    packet.put_slice(&[0xff; 8]);

    let epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as u32;
    packet.put_u32(epoch);
    packet.put_slice(&[0xff; 16]);
    packet
}

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:8053").expect("Could not bind to address");
    println!("Dummycloud is now listening");

    loop {
        let mut buf = [0; 1024];
        let (amt, src) = socket.recv_from(&mut buf)?;
        println!("connected from: {} with a message of length: {}", src, amt);

        let buf = &mut buf[..amt];

        socket.send_to(create_timesync_packet().bytes(), &src)?;
    }
}
