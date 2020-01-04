use crypto::aes::{cbc_decryptor, cbc_encryptor, KeySize};
use crypto::blockmodes::{NoPadding, PkcsPadding};
use crypto::buffer::{RefReadBuffer, RefWriteBuffer, WriteBuffer};
use crypto::digest::Digest;
use crypto::md5::Md5;
use std::str;
use std::string::String;
use std::time::SystemTime;

use bytes::BufMut;

#[derive(Clone)]
pub struct UDPCodec {
    pub token: String,
    pub token_key: [u8; 16],
    pub token_iv: [u8; 16],
}

impl UDPCodec {
    pub fn new(token: &str) -> UDPCodec {
        let mut cloud_md5er = Md5::new();
        cloud_md5er.input_str(&token);

        let mut token_key: [u8; 16] = [0; 16];
        cloud_md5er.result(&mut token_key);

        let mut cloud_md5er = Md5::new();
        cloud_md5er.input(&token_key);

        cloud_md5er.input_str(&token);
        let mut token_iv: [u8; 16] = [0; 16];
        cloud_md5er.result(&mut token_iv);
        UDPCodec {
            token: token.to_string(),
            token_key: token_key,
            token_iv: token_iv,
        }
    }

    pub fn decode_response(&self, header: &[u8], encrypted_body: &[u8]) -> Option<String> {
        if encrypted_body.len() == 0 {
            return None;
        }
        let mut digester = Md5::new();
        digester.input(&header[..16]);
        digester.input_str(&self.token);
        digester.input(encrypted_body);

        let mut digest: [u8; 16] = [0; 16];
        digester.result(&mut digest);

        let checksum = &header[16..];

        if !checksum.iter().zip(&digest).all(|(a, b)| a == b) {
            println!("not equal checksums");
            return None;
        }
        println!("checksums are equal!");
        let mut decipherer = cbc_decryptor(
            KeySize::KeySize128,
            &self.token_key,
            &self.token_iv,
            NoPadding,
        );

        let mut underlying_buffer = vec![0; encrypted_body.len()];
        decipherer
            .decrypt(
                &mut RefReadBuffer::new(&encrypted_body),
                &mut RefWriteBuffer::new(&mut underlying_buffer),
                true,
            )
            .expect("something went horribly wrong decrypting");

        let output = match String::from_utf8(underlying_buffer) {
            Ok(s) => s,
            Err(_) => String::from("{}\0"),
        };
        let output = output.split('\0').next().unwrap();
        Some(String::from(output))
    }

    pub fn encode_response(&self, message: &Vec<u8>, device_id: u32) -> Vec<u8> {
        let mut packet: Vec<u8> = vec![];
        // byte 0: write header
        packet.push(0x21);
        packet.push(0x31);

        //byte 2: size of encrypted body
        let mut cipherer = cbc_encryptor(
            KeySize::KeySize128,
            &self.token_key,
            &self.token_iv,
            PkcsPadding,
        );
        // Need to give the write buffer enough space to add padding bytes for AES-CBC encryption
        let mut underlying_buffer = vec![0; message.len().next_power_of_two()];

        // do the encryption, and get the number of bytes written in the underlying buffer
        let encrypted_size: usize = {
            let mut read_buffer = &mut RefReadBuffer::new(&message);
            let mut write_buffer = RefWriteBuffer::new(&mut underlying_buffer);
            cipherer
                .encrypt(&mut read_buffer, &mut write_buffer, true)
                .unwrap();
            write_buffer.position()
        };
        let encrypted_body = &underlying_buffer[..encrypted_size];
        packet.put_u16(32 + encrypted_size as u16);

        // byte 4: write nothing
        packet.put_u32(0);

        // byte 8
        packet.put_u32(device_id);

        // byte 12: write current epoch time
        let epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        packet.put_u32(epoch + 1);

        assert!(packet.len() == 16, "The packet should have 16 bytes by now");

        //byte 16: md5 hash of the first 16 bytes of header, the token and the encrypted body
        let mut digester = Md5::new();
        digester.input(&packet[..16]);
        digester.input_str(&self.token);
        digester.input(encrypted_body);

        let mut digest: [u8; 16] = [0; 16];
        digester.result(&mut digest);
        packet.put_slice(&digest);

        assert!(packet.len() == 32, "The packet should have 32 bytes by now");

        // byte 32: the rest of the encrypted body
        packet.extend(encrypted_body);
        packet
    }
}
