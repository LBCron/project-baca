//! Pseudonymous fields for security events. A log line carries a keyed hash
//! instead of an email address and the client's network instead of its
//! address, so attempts on one account or from one network still correlate
//! while the log names neither.

use crate::middleware::rate_limit::rate_subject;
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use std::net::Ipv4Addr;

/// First 16 hex characters of HMAC-SHA256 over the trimmed, lowercased
/// address, keyed with the JWT secret: stable for one address until the
/// secret rotates, and not reversible by hashing a list of addresses
/// without the key.
pub fn email_hash(key: &str, email: &str) -> String {
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(key.as_bytes()) else {
        return String::new();
    };
    mac.update(b"log.email:");
    mac.update(email.trim().to_lowercase().as_bytes());
    mac.finalize()
        .into_bytes()
        .iter()
        .take(8)
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The /24 of an IPv4 client or the /64 of an IPv6 one.
pub fn client_prefix(ip: &str) -> String {
    let subject = rate_subject(ip);
    match subject.parse::<Ipv4Addr>() {
        Ok(v4) => {
            let [a, b, c, _] = v4.octets();
            format!("{a}.{b}.{c}.0/24")
        }
        Err(_) => subject,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_hash_is_keyed_stable_and_case_blind() {
        let h = email_hash("k1", "Reader@Example.com ");
        assert_eq!(h.len(), 16);
        assert_eq!(h, email_hash("k1", "reader@example.com"));
        assert_ne!(h, email_hash("k2", "reader@example.com"));
        assert_ne!(h, email_hash("k1", "other@example.com"));
        assert!(!h.contains("reader"));
    }

    #[test]
    fn client_prefix_keeps_only_the_network() {
        assert_eq!(client_prefix("203.0.113.77"), "203.0.113.0/24");
        assert_eq!(client_prefix("::ffff:203.0.113.77"), "203.0.113.0/24");
        assert_eq!(
            client_prefix("2001:db8:aa:bb:1:2:3:4"),
            "2001:db8:aa:bb::/64"
        );
    }
}
