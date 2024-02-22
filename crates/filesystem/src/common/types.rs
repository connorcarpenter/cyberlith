
use crate::common::{ReadRequest, ReadResponse, WriteRequest, WriteResponse};

pub enum Request {
    Read(ReadRequest),
    Write(WriteRequest),
}

pub enum Response {
    Read(ReadResponse),
    Write(WriteResponse),
}