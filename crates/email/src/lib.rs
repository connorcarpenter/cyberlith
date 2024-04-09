use lettre::{SmtpTransport, Transport, transport::smtp::authentication::Credentials};
use lettre::message::Body;

pub fn send(
    sender_email: &str,
    recipient_email: &str,
    subject: &str,
    text_content: &str,
    html_content: &str
) -> Result<(), String> {

    let text_content = text_content.to_string();
    let text_content = Body::new(text_content);
    let html_content = html_content.to_string();
    let html_content = Body::new(html_content);

    // build email
    let email = lettre::Message::builder()
        .from(
            sender_email
                .parse()
                .unwrap(),
        )
        .to(recipient_email
            .parse()
            .unwrap())
        .subject(subject)
        .multipart(
            lettre::message::MultiPart::alternative()
                .singlepart(
                    lettre::message::SinglePart::builder()
                        .header(lettre::message::header::ContentType::TEXT_PLAIN)
                        .body(text_content),
                )
                .singlepart(
                    lettre::message::SinglePart::builder()
                        .header(lettre::message::header::ContentType::TEXT_HTML)
                        .body(html_content),
                ),
        )
        .unwrap();

    // Create SMTP client credentials using username and password
    let api_key = include_str!("../../../.secrets/sendgrid_api_key");
    let creds = Credentials::new("apikey".to_string(), api_key.to_string());

    // Open a secure connection to the SMTP server using STARTTLS
    let mailer = SmtpTransport::starttls_relay("smtp.sendgrid.net")
        .unwrap()  // Unwrap the Result, panics in case of error
        .credentials(creds)  // Provide the credentials to the transport
        .build();  // Construct the transport

    // Send the email
    mailer.send(&email).unwrap();

    Ok(())
}