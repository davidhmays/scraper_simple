// src/mailer.rs

use reqwest::blocking::Client;
use serde::Serialize;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum MailerError {
    RequestFailed(String),
    ApiError(String),
}

impl fmt::Display for MailerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MailerError::RequestFailed(msg) => write!(f, "Request failed: {}", msg),
            MailerError::ApiError(msg) => write!(f, "API error: {}", msg),
        }
    }
}

impl Error for MailerError {}

pub struct BrevoMailer {
    api_key: String,
    sender_email: String,
    sender_name: String,
    client: Client,
}

#[derive(Serialize)]
struct BrevoSender<'a> {
    name: &'a str,
    email: &'a str,
}

#[derive(Serialize)]
struct BrevoRecipient<'a> {
    email: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BrevoPayload<'a> {
    sender: BrevoSender<'a>,
    to: Vec<BrevoRecipient<'a>>,
    subject: &'a str,
    html_content: String,
}

impl BrevoMailer {
    pub fn new(api_key: String, sender_email: String, sender_name: String) -> Self {
        Self {
            api_key,
            sender_email,
            sender_name,
            client: Client::new(),
        }
    }

    pub fn send_magic_link(
        &self,
        recipient_email: &str,
        magic_link: &str,
    ) -> Result<(), MailerError> {
        let subject = "Your Magic Sign-In Link";
        let html_content = format!(
            r#"
            <h1>Sign In to Scraper Simple</h1>
            <p>Click the link below to sign in to your account. This link will expire in 15 minutes.</p>
            <p><a href="{}">Click here to sign in</a></p>
            <p>If you did not request this link, you can safely ignore this email.</p>
        "#,
            magic_link
        );

        let payload = BrevoPayload {
            sender: BrevoSender {
                name: &self.sender_name,
                email: &self.sender_email,
            },
            to: vec![BrevoRecipient {
                email: recipient_email,
            }],
            subject,
            html_content,
        };

        let resp = self
            .client
            .post("https://api.brevo.com/v3/smtp/email")
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .map_err(|e| MailerError::RequestFailed(e.to_string()))?;

        if !resp.status().is_success() {
            let error_body = resp.text().unwrap_or_else(|_| "Unknown error".to_string());
            return Err(MailerError::ApiError(format!(
                "Failed to send email: {}",
                error_body
            )));
        }

        Ok(())
    }
}
