use naia_serde::{BitReader, SerdeInternal};

use http::{ClientHttpRequest, ClientHttpResponse, HttpResponse};

use crate::{LoginRequest, LoginResponse};

impl ClientHttpRequest for LoginRequest {
    type Response = LoginResponse;
}

impl ClientHttpResponse for LoginResponse {}

impl From<HttpResponse> for LoginResponse {
    fn from(response: HttpResponse) -> Self {
        if !response.ok() {
            panic!("response not ok ... handle this!");
        }
        let bytes = response.bytes();
        let mut bit_reader = BitReader::new(&bytes);
        let new_self = Self::de(&mut bit_reader).unwrap(); // handle this later!
        new_self
    }
}