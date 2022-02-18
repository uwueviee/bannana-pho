use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub async fn verify_token(secret: String, nonce: Option<String>, token: String) -> bool {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("Failed to load key for hmac verification!");

    mac.update(nonce.expect("Missing nonce?").as_bytes());

    mac.verify_slice(hex::decode(token).expect("Failed to get token as bytes!").as_slice()).is_ok()
}