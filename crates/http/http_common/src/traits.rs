use naia_serde::{BitReader, FileBitWriter, Serde};

use crate::{Method, Request, Response, ResponseError};

pub trait ApiRequest: Serde + 'static {
    type Response: ApiResponse;

    fn method() -> Method;

    fn path() -> &'static str;

    fn endpoint_key() -> String {
        format!("{} /{}", Self::method().as_str(), Self::path())
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();
        self.ser(&mut bit_writer);
        bit_writer.to_bytes()
    }

    fn to_request(&self, addr: &str, port: u16) -> Request {
        let url = format!("http://{}:{}/{}", addr, port, Self::path());
        let bytes = self.to_bytes().to_vec();
        Request::new(Self::method(), &url, bytes)
    }

    fn from_request(request: Request) -> Result<Self, ()> {
        let bytes = request.body;
        let mut bit_reader = BitReader::new(&bytes);
        match Self::de(&mut bit_reader) {
            Ok(request) => Ok(request),
            Err(_) => Err(()),
        }
    }
}

pub trait ApiResponse: Serde {
    fn to_response(&self) -> Response {
        let mut bit_writer = FileBitWriter::new();
        self.ser(&mut bit_writer);
        let bytes = bit_writer.to_bytes().to_vec();

        let mut response = Response::default();
        response.body = bytes;

        response
    }

    fn from_response(response: Response) -> Result<Self, ResponseError> {
        let bytes = response.body;
        let mut bit_reader = BitReader::new(&bytes);
        match Self::de(&mut bit_reader) {
            Ok(response) => Ok(response),
            Err(_) => Err(ResponseError::SerdeError),
        }
    }
}
