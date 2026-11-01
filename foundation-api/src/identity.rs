//! Google ID token verification. This module is not a login endpoint.
//!
//! The caller must obtain keys from Google's trusted HTTPS JWKS endpoint and
//! compare a one-use server-generated nonce bound to the pending sign-in.
//! Token verification alone must never create a session.
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq)]
pub struct VerifiedIdentity {
    pub issuer: String,
    pub subject: String,
    pub email: String,
    pub name: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Audience {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Deserialize)]
struct Claims {
    iss: String,
    sub: String,
    aud: Audience,
    azp: Option<String>,
    email: String,
    email_verified: bool,
    name: Option<String>,
    nonce: String,
}

/// An RSA public key selected by its `kid` from a trusted Google JWKS response.
/// Never accept keys supplied by a client or by an untrusted token header URL.
pub struct TrustedKey<'a> {
    pub key_id: &'a str,
    pub key: &'a DecodingKey,
}

#[derive(Debug, PartialEq, Eq)]
pub enum VerificationError {
    InvalidConfiguration,
    InvalidToken,
}

/// Validates a Google ID token against a trusted key and a one-use nonce.
/// Nonce consumption and replay prevention must be performed transactionally by the caller.
pub fn verify_google_id_token(
    token: &str,
    client_id: &str,
    expected_nonce: &str,
    key: TrustedKey<'_>,
) -> Result<VerifiedIdentity, VerificationError> {
    if client_id.is_empty() || expected_nonce.is_empty() || key.key_id.is_empty() {
        return Err(VerificationError::InvalidConfiguration);
    }
    if token.is_empty() || token.len() > 16_384 {
        return Err(VerificationError::InvalidToken);
    }
    let header = decode_header(token).map_err(|_| VerificationError::InvalidToken)?;
    if header.alg != Algorithm::RS256 || header.kid.as_deref() != Some(key.key_id) {
        return Err(VerificationError::InvalidToken);
    }
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[client_id]);
    validation.set_issuer(&["https://accounts.google.com", "accounts.google.com"]);
    validation.leeway = 0;
    let claims = decode::<Claims>(token, key.key, &validation)
        .map_err(|_| VerificationError::InvalidToken)?
        .claims;
    let audience_valid = match &claims.aud {
        Audience::Single(aud) => aud == client_id,
        Audience::Multiple(aud) => !aud.is_empty() && aud.iter().any(|a| a == client_id),
    };
    let requires_azp = matches!(&claims.aud, Audience::Multiple(aud) if aud.len() > 1);
    if !["https://accounts.google.com", "accounts.google.com"].contains(&claims.iss.as_str())
        || !audience_valid
        || (requires_azp && claims.azp.as_deref() != Some(client_id))
        || claims.azp.as_deref().is_some_and(|azp| azp != client_id)
        || !claims.email_verified
        || claims.sub.is_empty()
        || claims.sub.len() > 255
        || claims.email.is_empty()
        || claims.email.len() > 320
        || claims.nonce != expected_nonce
    {
        return Err(VerificationError::InvalidToken);
    }
    Ok(VerifiedIdentity {
        // Google documents both issuer spellings as equivalent.
        issuer: "https://accounts.google.com".to_string(),
        subject: claims.sub,
        email: claims.email,
        name: claims.name.unwrap_or_default(),
    })
}
