// from naia
use naia_socket_shared::Random;

/// returns a random f32 value between an upper & lower bound
pub fn gen_range_f32(lower: f32, upper: f32) -> f32 {
    Random::gen_range_f32(lower, upper)
}

/// returns a random u32 value between an upper & lower bound
pub fn gen_range_u32(lower: u32, upper: u32) -> u32 {
    Random::gen_range_u32(lower, upper)
}

/// returns a random boolean value between an upper & lower bound
pub fn gen_bool() -> bool {
    Random::gen_bool()
}

// direct from rand
use rand::{seq::SliceRandom, thread_rng, Rng};

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

// shuffling vec
pub fn shuffle_vec<T>(vec: &mut Vec<T>) {
    let mut rng = thread_rng();
    vec.shuffle(&mut rng);
}
