mod server;
pub use server::*;

pub use http_common::{Method, Request, Response};

pub mod async_dup {
    pub use async_dup::*;
}

pub mod smol {
    pub use smol::*;
}

pub mod http_log_util {

    use http_common::ResponseError;

    pub fn send_req(
        self_service_sender_name: &str,
        othe_service_recver_name: &str,
        request_name: &str,
    ) {
        logging::info!(
            "{} -> {}: {} request",
            self_service_sender_name,
            othe_service_recver_name,
            request_name
        );
    }

    pub fn recv_req(
        self_service_recver_name: &str,
        othe_service_sender_name: &str,
        response_name: &str,
    ) {
        logging::info!(
            "{} <- {}: {} request",
            self_service_recver_name,
            othe_service_sender_name,
            response_name
        );
    }

    pub fn send_res(
        self_service_sender_name: &str,
        othe_service_recver_name: &str,
        request_name: &str,
    ) {
        logging::info!(
            "{} -> {}: {} response",
            self_service_sender_name,
            othe_service_recver_name,
            request_name
        );
    }

    pub fn recv_res(
        self_service_recver_name: &str,
        othe_service_sender_name: &str,
        response_name: &str,
    ) {
        logging::info!(
            "{} <- {}: {} response",
            self_service_recver_name,
            othe_service_sender_name,
            response_name
        );
    }

    pub fn fail_recv_res<T>(
        self_service_recver_name: &str,
        othe_service_sender_name: &str,
        response_name: &str,
    ) -> Result<T, ResponseError> {
        let error_msg = format!(
            "{} <- {}: {} response FAIL!",
            self_service_recver_name, othe_service_sender_name, response_name
        );
        logging::warn!("{}", error_msg);
        Err(ResponseError::InternalServerError(error_msg))
    }
}
