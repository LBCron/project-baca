//! Transactional email dispatch for verification OTPs over SMTP (lettre).
//!
//! STARTTLS with authentication for real relays, cleartext for the local
//! Mailpit; either way a failed hand-off surfaces as an error so signup
//! never reports a code that never left the server.

use crate::config::{EmailConfig, SmtpSecurity};
use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use shared::AppError;
use std::time::Duration;
use tracing::{info, warn};

const SMTP_TIMEOUT: Duration = Duration::from_secs(5);

/// Email with its local part hidden, for log lines: `a***@example.com`.
pub fn mask_email(email: &str) -> String {
    match email.split_once('@') {
        Some((local, domain)) => {
            let first = local.chars().next().map(String::from).unwrap_or_default();
            format!("{first}***@{domain}")
        }
        None => "***".to_string(),
    }
}

fn smtp_error(step: &str, err: impl std::fmt::Display) -> AppError {
    AppError::ExternalService(format!("SMTP {step} failed: {err}"))
}

fn build_transport(config: &EmailConfig) -> Result<AsyncSmtpTransport<Tokio1Executor>, AppError> {
    let builder = match config.smtp_security {
        SmtpSecurity::StartTls => {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_host)
                .map_err(|e| smtp_error("TLS setup", e))?
        }
        SmtpSecurity::None => {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.smtp_host)
        }
    }
    .port(config.smtp_port)
    .timeout(Some(SMTP_TIMEOUT));
    let builder = if config.smtp_user.is_empty() {
        builder
    } else {
        builder.credentials(Credentials::new(
            config.smtp_user.clone(),
            config.smtp_password.clone(),
        ))
    };
    Ok(builder.build())
}

fn otp_message(config: &EmailConfig, to_email: &str, otp: &str) -> Result<Message, AppError> {
    let from_address: Address = config
        .smtp_from_email
        .parse()
        .map_err(|e| smtp_error("sender address", e))?;
    let to_address: Address = to_email
        .parse()
        .map_err(|e| smtp_error("recipient address", e))?;
    Message::builder()
        .from(Mailbox::new(
            Some(config.smtp_from_name.clone()),
            from_address,
        ))
        .to(Mailbox::new(None, to_address))
        .subject("Project Baca Verification Code")
        .header(ContentType::TEXT_PLAIN)
        .body(format!(
            "Welcome to Project Baca!\r\n\r\nYour 6-digit verification code is:\r\n\r\n  {otp}\r\n\r\nThis code will expire in 10 minutes.\r\n"
        ))
        .map_err(|e| smtp_error("message build", e))
}

/// Sends an OTP verification email. Fails closed: a code that never left the
/// server must surface as an error, or the caller reports success and the
/// user is stranded at the OTP step.
pub async fn send_otp_email(
    config: &EmailConfig,
    to_email: &str,
    otp: &str,
) -> Result<(), AppError> {
    let relay = format!("{}:{}", config.smtp_host, config.smtp_port);
    let recipient = mask_email(to_email);
    info!(
        "Dispatching OTP email to {recipient} via {relay} ({:?})",
        config.smtp_security
    );

    let transport = build_transport(config)?;
    let message = otp_message(config, to_email, otp)?;
    transport.send(message).await.map_err(|e| {
        warn!("OTP email to {recipient} via {relay} failed: {e}");
        smtp_error("delivery", e)
    })?;

    info!("Dispatched verification OTP email to {recipient}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_email_hides_the_local_part() {
        assert_eq!(mask_email("alice@example.com"), "a***@example.com");
        assert_eq!(mask_email("x@baca.local"), "x***@baca.local");
        assert_eq!(mask_email("not-an-email"), "***");
    }

    #[test]
    fn otp_message_addresses_sender_and_recipient() {
        let config = EmailConfig::default();
        let message = otp_message(&config, "reader@example.com", "123456").expect("builds");
        let envelope = message.envelope();
        assert_eq!(
            envelope.from().map(ToString::to_string).as_deref(),
            Some(config.smtp_from_email.as_str())
        );
        assert_eq!(
            envelope
                .to()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["reader@example.com".to_string()]
        );
        let raw = String::from_utf8(message.formatted()).expect("utf8");
        assert!(raw.contains("123456"));
        assert!(raw.contains("Subject: Project Baca Verification Code"));
        assert!(otp_message(&config, "not an address", "123456").is_err());
    }

    /// Cleartext delivery to the dev Mailpit; skipped when no relay listens
    /// on 1025 so the unit suite stays service-free.
    #[tokio::test]
    async fn mailpit_accepts_cleartext_delivery_when_running() {
        if tokio::net::TcpStream::connect("127.0.0.1:1025")
            .await
            .is_err()
        {
            return;
        }
        let config = EmailConfig {
            smtp_host: "localhost".to_string(),
            smtp_port: 1025,
            smtp_security: SmtpSecurity::None,
            ..EmailConfig::default()
        };
        send_otp_email(&config, "unit-test@example.com", "123456")
            .await
            .expect("Mailpit must accept a cleartext OTP mail");
    }

    #[tokio::test]
    async fn unreachable_relay_is_an_external_service_error() {
        let config = EmailConfig {
            smtp_host: "127.0.0.1".to_string(),
            smtp_port: 1,
            smtp_security: SmtpSecurity::None,
            ..EmailConfig::default()
        };
        let err = send_otp_email(&config, "reader@example.com", "123456")
            .await
            .expect_err("closed port must fail");
        assert!(matches!(err, AppError::ExternalService(_)), "{err:?}");
    }
}
