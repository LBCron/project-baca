//! Asynchronous transactional email dispatch for verification OTPs (Mailpit/SMTP).

use crate::config::EmailConfig;
use shared::AppError;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{info, warn};

/// One SMTP exchange: optional write, then a reply whose 2xx/3xx code is
/// enforced. A silent failure here would report "Successfully dispatched"
/// for an email that never left the server.
async fn smtp_step(
    stream: &mut TcpStream,
    buf: &mut [u8],
    step: &str,
    payload: Option<&[u8]>,
) -> Result<(), AppError> {
    if let Some(bytes) = payload {
        timeout(Duration::from_secs(3), stream.write_all(bytes))
            .await
            .map_err(|_| AppError::ExternalService(format!("SMTP {step} timed out")))?
            .map_err(|e| AppError::ExternalService(format!("SMTP {step} failed: {e}")))?;
    }
    let n = timeout(Duration::from_secs(3), stream.read(buf))
        .await
        .map_err(|_| AppError::ExternalService(format!("SMTP {step} timed out")))?
        .map_err(|e| AppError::ExternalService(format!("SMTP {step} failed: {e}")))?;
    let reply = String::from_utf8_lossy(&buf[..n]);
    let code = reply.get(0..3).unwrap_or("");
    if !(code.starts_with('2') || code.starts_with('3')) {
        return Err(AppError::ExternalService(format!(
            "SMTP {step} rejected: {}",
            reply.lines().next().unwrap_or("unknown")
        )));
    }
    Ok(())
}

/// Sends an OTP verification email to the user via SMTP (local Mailpit)
pub async fn send_otp_email(
    config: &EmailConfig,
    to_email: &str,
    otp: &str,
) -> Result<(), AppError> {
    let addr = format!("{}:{}", config.smtp_host, config.smtp_port);
    info!(
        "Dispatching OTP email for {} via SMTP at {}",
        to_email, addr
    );

    let connect_future = TcpStream::connect(&addr);
    let mut stream = match timeout(Duration::from_secs(3), connect_future).await {
        Ok(Ok(stream)) => stream,
        Ok(Err(e)) => {
            warn!("SMTP connection to {addr} failed: {e}. Falling back to log trace.");
            info!("MOCK EMAIL DISPATCH: To: {to_email} (OTP redacted)");
            return Ok(());
        }
        Err(_) => {
            warn!("SMTP connection to {addr} timed out. Falling back to log trace.");
            info!("MOCK EMAIL DISPATCH: To: {to_email} (OTP redacted)");
            return Ok(());
        }
    };

    let mut buf = [0u8; 1024];

    smtp_step(&mut stream, &mut buf, "greeting", None).await?;
    smtp_step(&mut stream, &mut buf, "EHLO", Some(b"EHLO localhost\r\n")).await?;
    let mail_from = format!("MAIL FROM:<{}>\r\n", config.smtp_from_email);
    smtp_step(
        &mut stream,
        &mut buf,
        "MAIL FROM",
        Some(mail_from.as_bytes()),
    )
    .await?;
    let rcpt_to = format!("RCPT TO:<{to_email}>\r\n");
    smtp_step(&mut stream, &mut buf, "RCPT TO", Some(rcpt_to.as_bytes())).await?;
    smtp_step(&mut stream, &mut buf, "DATA", Some(b"DATA\r\n")).await?;
    let email_body = format!(
        "From: {} <{}>\r\nTo: <{}>\r\nSubject: Project Baca Verification Code\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nWelcome to Project Baca!\r\n\r\nYour 6-digit verification code is:\r\n\r\n  {}\r\n\r\nThis code will expire in 10 minutes.\r\n.\r\n",
        config.smtp_from_name, config.smtp_from_email, to_email, otp
    );
    smtp_step(
        &mut stream,
        &mut buf,
        "payload",
        Some(email_body.as_bytes()),
    )
    .await?;
    let _ = stream.write_all(b"QUIT\r\n").await;

    info!(
        "Successfully dispatched verification OTP email to {}",
        to_email
    );
    Ok(())
}
