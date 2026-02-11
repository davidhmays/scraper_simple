use crate::errors::ServerError;
use serde_json::json;

pub struct BrevoMailer {
    api_key: String,
    sender_email: String,
    sender_name: String,
}

impl BrevoMailer {
    pub fn new(api_key: String, sender_email: String, sender_name: String) -> Self {
        Self {
            api_key,
            sender_email,
            sender_name,
        }
    }

    pub fn send_magic_link(&self, to_email: &str, magic_link: &str) -> Result<(), ServerError> {
        let client = reqwest::blocking::Client::new();

        let subject = "Log in to Scraper Simple";
        let html_content = format!(
            r#"
            <html>
                <body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
                    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
                        <h2>Welcome back!</h2>
                        <p>Click the link below to sign in to your account:</p>
                        <p style="margin: 25px 0;">
                            <a href="{link}" style="background-color: #007bff; color: white; padding: 10px 20px; text-decoration: none; border-radius: 5px; display: inline-block;">
                                Sign In
                            </a>
                        </p>
                        <p style="font-size: 0.9em; color: #666;">
                            Or copy and paste this link into your browser:<br>
                            <a href="{link}" style="color: #007bff;">{link}</a>
                        </p>
                        <hr style="margin-top: 30px; border: none; border-top: 1px solid #eee;">
                        <p style="font-size: 0.8em; color: #999;">
                            If you didn't request this login link, you can safely ignore this email.
                        </p>
                    </div>
                </body>
            </html>
            "#,
            link = magic_link
        );

        let body = json!({
            "sender": {
                "name": self.sender_name,
                "email": self.sender_email
            },
            "to": [
                {
                    "email": to_email
                }
            ],
            "subject": subject,
            "htmlContent": html_content
        });

        // Using Brevo's v3 API endpoint for transactional emails
        let response = client
            .post("https://api.brevo.com/v3/smtp/email")
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| ServerError::BadRequest(format!("Failed to send email request: {}", e)))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().unwrap_or_else(|_| "(no body)".to_string());
            Err(ServerError::BadRequest(format!(
                "Brevo API error: {} - {}",
                status, text
            )))
        }
    }
}
