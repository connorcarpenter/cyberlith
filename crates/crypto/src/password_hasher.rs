use bcrypt::{hash, verify as bverify, DEFAULT_COST};

pub fn process(password: &str) -> Result<String, ()> {
    match hash(password, DEFAULT_COST) {
        Ok(h) => Ok(h),
        Err(_) => Err(()),
    }
}

pub fn verify(a: &str, b: &str) -> bool {
    return bverify(a, b).unwrap();
}
