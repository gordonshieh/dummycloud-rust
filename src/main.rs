extern crate bytes;
extern crate crypto;
extern crate serde;
extern crate serde_json;

use std::ascii::escape_default;
use std::net::UdpSocket;
use std::str;
use std::time::SystemTime;

use bytes::{Buf, BufMut, BytesMut};
use serde_json::json;

mod codec;
mod payload;

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

    let epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
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

        let cloud_key = "replace me with cli params";
        let c = codec::UDPCodec::new(&cloud_key);

        // truncate the size of the buffer to appropriately handle later
        let buf = &buf[..amt];

        let header = &buf[..32];
        let encrypted_body = &buf[32..];
        let stamp = (&header[12..]).get_u32();
        println!("stamp: {}", stamp);
        let device_id = (&header[8..]).get_u32();
        let response = match c.decode_response(header, encrypted_body) {
            Some(s) => s,
            None => {
                if stamp == 0 {
                    println!("Robot connected!");
                    socket.send_to(create_timesync_packet().bytes(), &src)?;
                } else {
                    socket.send_to(&buf, &src)?;
                }
                continue;
            }
        };

        let response = payload::parse_json(&response);
        let reply_json: payload::ResponsePayload = match response.method.as_str() {
            "_otc.info" => {
                println!("_otc.info");

                payload::ResponsePayload::new(
                    response.id,
                    json!({
                        "otc_list": [{
                            "ip": "130.83.47.181",
                            "port": 8053
                        }
                        ],
                        "otc_test": {
                            "list": [{
                                "ip": "130.83.47.181",
                                "port": 8053
                            }
                            ],
                            "interval": 1800,
                            "firsttest": 1193
                        }
                    }),
                )
            }
            _ => payload::ResponsePayload::new(response.id, serde_json::to_value("ok")?),
        };
        let reply = c.encode_response(&serde_json::to_vec(&reply_json)?, device_id);
        socket.send_to(&reply, &src)?;
    }
}
