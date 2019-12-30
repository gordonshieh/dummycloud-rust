
use crypto::md5::Md5;
use crypto::digest::Digest;

#[derive(Clone)]
pub struct UDPCodec {
    pub token: String,
    pub token_key: String,
    pub token_iv: String
}

impl UDPCodec {
    pub fn new(token: &str) -> UDPCodec {
        let mut cloud_md5er = Md5::new();
        cloud_md5er.input_str(&token);
        let token_key = cloud_md5er.result_str();

        let mut cloud_md5er = Md5::new();
        cloud_md5er.input_str(&token_key);
        cloud_md5er.input_str(&token);
        let token_iv = cloud_md5er.result_str();
        UDPCodec {
            token: token.to_string(),
            token_key: token_key,
            token_iv: token_iv
        }
    }
}