use ring::{signature, signature::KeyPair};

pub struct PrivateKey {
    bytes: Vec<u8>,
}

impl PrivateKey {
    pub fn new() -> Self {
        let rng = ring::rand::SystemRandom::new();
        let bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)
            .expect("Failed to generate private key")
            .as_ref()
            .to_vec();
        Self {
            bytes
        }
    }

    pub fn sign(&self, message: &[u8]) -> Signature {
        let key_pair = signature::Ed25519KeyPair::from_pkcs8(&self.bytes)
            .expect("Failed to extract private key");
        Signature { bytes: key_pair.sign(message).as_ref().to_vec() }
    }

    pub fn decrypt(&self, _message: &[u8]) -> Vec<u8> {
        todo!()
    }
}

pub struct Signature {
    bytes: Vec<u8>,
}

pub struct PublicKey {
    bytes: Vec<u8>,
}

impl PublicKey {
    pub fn new(private_key: &PrivateKey) -> Self {
        let bytes = signature::Ed25519KeyPair::from_pkcs8(&private_key.bytes)
            .expect("Failed to extract public key")
            .public_key().as_ref().to_vec();
        Self {
            bytes
        }
    }

    pub fn from(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.to_vec()
        }
    }

    pub fn verify(&self, message: &[u8], signature: &Signature) -> bool {
        let public_key = signature::UnparsedPublicKey::new(&signature::ED25519, &self.bytes);
        public_key.verify(message, &signature.bytes).is_ok()
    }

    pub fn encrypt(&self, _message: &[u8]) -> Vec<u8> {
        todo!()
    }
}

pub fn generate_public_private_keys() -> (PrivateKey, PublicKey) {
    let private_key = PrivateKey::new();
    let public_key = PublicKey::new(&private_key);
    (private_key, public_key)
}