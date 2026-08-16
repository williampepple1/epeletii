//! Email service integration using Resend API.

use serde::Serialize;

#[derive(Serialize)]
struct ResendEmailRequest {
    from: String,
    to: Vec<String>,
    subject: String,
    html: String,
}

#[derive(Clone)]
pub struct EmailService {
    api_key: String,
    from_address: String,
    client: reqwest::Client,
}

impl EmailService {
    pub fn new() -> Self {
        let api_key = std::env::var("RESEND_API_KEY")
            .unwrap_or_default();
        let from_address = std::env::var("RESEND_FROM")
            .unwrap_or_else(|_| "Epeletii <onboarding@resend.dev>".to_string());
        Self {
            api_key,
            from_address,
            client: reqwest::Client::new(),
        }
    }

    pub async fn send_welcome_email(&self, to_email: &str, display_name: &str) -> Result<(), String> {
        let subject = "Welcome to Epeletii — Ibani Scrabble! 🦛";
        let html = format!(
            r#"<div style="font-family: sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; border: 1px solid #e5e5e5; border-radius: 12px;">
                <h2 style="color: #d97706; margin-bottom: 10px;">Welcome, {}! 🦛</h2>
                <p style="font-size: 16px; line-height: 1.5; color: #374151;">
                    We are thrilled to welcome you to <strong>Epeletii</strong>, the official online home of Ibani Scrabble!
                </p>
                <p style="font-size: 16px; line-height: 1.5; color: #374151;">
                    Epeletii is designed to help preserve and celebrate the beautiful Ibani language. Whether you're playing with friends or testing your vocabulary against opponents, we hope you have an incredible time.
                </p>
                <div style="margin: 25px 0;">
                    <a href="https://game.ibani.online" style="background-color: #d97706; color: white; padding: 12px 24px; text-decoration: none; border-radius: 8px; font-weight: bold; display: inline-block;">Start Playing Now</a>
                </div>
                <p style="font-size: 14px; color: #6b7280; margin-top: 30px; border-top: 1px solid #e5e5e5; padding-top: 15px;">
                    If you didn't sign up for Epeletii, please ignore this email.
                </p>
            </div>"#,
            display_name
        );

        self.send_email(to_email, subject, &html).await
    }

    pub async fn send_reset_password_email(&self, to_email: &str, token: &str) -> Result<(), String> {
        let subject = "Reset Your Epeletii Password 🔑";
        let reset_link = format!(
            "https://game.ibani.online/?action=reset-password&token={}&email={}",
            token,
            to_email
        );
        let html = format!(
            r#"<div style="font-family: sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; border: 1px solid #e5e5e5; border-radius: 12px;">
                <h2 style="color: #d97706; margin-bottom: 10px;">Reset Your Password 🔑</h2>
                <p style="font-size: 16px; line-height: 1.5; color: #374151;">
                    We received a request to reset your password for your Epeletii account.
                </p>
                <p style="font-size: 16px; line-height: 1.5; color: #374151;">
                    Please click the button below to choose a new password. This link is valid for 1 hour.
                </p>
                <div style="margin: 25px 0;">
                    <a href="{}" style="background-color: #d97706; color: white; padding: 12px 24px; text-decoration: none; border-radius: 8px; font-weight: bold; display: inline-block;">Reset Password</a>
                </div>
                <p style="font-size: 14px; color: #6b7280; margin-top: 30px; border-top: 1px solid #e5e5e5; padding-top: 15px;">
                    Link not working? Copy and paste this URL into your browser:<br/>
                    <a href="{}" style="color: #d97706; word-break: break-all;">{}</a>
                </p>
                <p style="font-size: 14px; color: #6b7280; margin-top: 15px;">
                    If you didn't request a password reset, you can safely ignore this email.
                </p>
            </div>"#,
            reset_link, reset_link, reset_link
        );

        self.send_email(to_email, subject, &html).await
    }

    async fn send_email(&self, to: &str, subject: &str, html: &str) -> Result<(), String> {
        let req_body = ResendEmailRequest {
            from: self.from_address.clone(),
            to: vec![to.to_string()],
            subject: subject.to_string(),
            html: html.to_string(),
        };

        let response = self.client
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .await
            .map_err(|e| format!("HTTP request error: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            log::error!("Resend API error (status {}): {}", status, body_text);
            return Err(format!("Resend returned error status {}: {}", status, body_text));
        }

        log::info!("Successfully sent email to {} via Resend", to);
        Ok(())
    }
}
