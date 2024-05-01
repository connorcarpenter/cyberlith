use http_common::Response;

pub struct CookieStore {

}

impl CookieStore {
    pub(crate) fn new() -> Self {
        Self {

        }
    }

    pub(crate) fn handle_response(&mut self, response: &Response) {
        unimplemented!()
    }
}