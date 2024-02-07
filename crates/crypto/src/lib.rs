mod private_public_keys;
mod u16_token;

pub use u16_token::U16Token;
pub use private_public_keys::{PrivateKey, PublicKey, Signature, generate_public_private_keys};

use rand::{thread_rng, Rng};

pub fn generate_random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"abcdefghjkmnpqrstuvwxyz123456789";

    let token: String = (0..length)
        .map(|_| {
            let idx = thread_rng().gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    token
}