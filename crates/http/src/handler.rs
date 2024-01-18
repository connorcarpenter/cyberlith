use naia_serde::{FileBitWriter, Serde};

use crate::HttpResponse;

pub trait ClientHttpRequest: Serde {
    type Response: ClientHttpResponse;

    fn to_bytes(&self) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();
        self.ser(&mut bit_writer);
        bit_writer.to_bytes()
    }
}

pub trait ClientHttpResponse: Serde + From<HttpResponse> {
}