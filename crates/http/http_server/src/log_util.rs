
pub fn send_req(
    self_service_sender_name: &str,
    other_service_recver_name: &str,
    request_name: &str,
) {
    logging::info!(
        "{} -> [{}] -> {}",
        self_service_sender_name,
        request_name,
        other_service_recver_name,
    );
}

pub fn recv_req(self_service_recver_name: &str, request_url: &str, request_name: &str) {
    logging::info!(
        "{} <- [{}] <- {}",
        self_service_recver_name,
        request_name,
        request_url
    );
}

pub fn send_res(self_service_sender_name: &str, response_name: &str) {
    logging::info!(
        "{} -> [{}]",
        self_service_sender_name,
        response_name
    );
}

pub fn recv_res(
    self_service_recver_name: &str,
    other_service_sender_name: &str,
    response_name: &str,
) {
    logging::info!(
        "{} <- [{}] <- {}",
        self_service_recver_name,
        response_name,
        other_service_sender_name,
    );
}
