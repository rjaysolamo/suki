use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, encode};
use serde_json::{Value, json};
use std::time::{SystemTime, UNIX_EPOCH};
use suki_foundation_api::identity::{TrustedKey, VerificationError, verify_google_id_token};

const PRIVATE: &[u8] = include_bytes!("fixtures/synthetic-rsa.pem");
const PUBLIC: &[u8] = include_bytes!("fixtures/synthetic-rsa-public.pem");

fn claims() -> Value {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    json!({"iss":"https://accounts.google.com", "sub":"google-subject", "aud":"mobile-client", "exp":now+600, "email":"test@example.invalid", "email_verified":true, "nonce":"server-issued-nonce", "name":"Test User"})
}
fn sign(claims: &Value, kid: &str) -> String {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.to_owned());
    encode(
        &header,
        claims,
        &EncodingKey::from_rsa_pem(PRIVATE).unwrap(),
    )
    .unwrap()
}
fn verify(token: &str, public: &[u8]) -> Result<(), VerificationError> {
    let key = DecodingKey::from_rsa_pem(public).unwrap();
    verify_google_id_token(
        token,
        "mobile-client",
        "server-issued-nonce",
        TrustedKey {
            key_id: "test-key",
            key: &key,
        },
    )
    .map(|_| ())
}

#[test]
fn accepts_signed_identity_and_normalizes_google_issuer() {
    let mut input = claims();
    input["iss"] = json!("accounts.google.com");
    let key = DecodingKey::from_rsa_pem(PUBLIC).unwrap();
    let token = sign(&input, "test-key");
    let identity = verify_google_id_token(
        &token,
        "mobile-client",
        "server-issued-nonce",
        TrustedKey {
            key_id: "test-key",
            key: &key,
        },
    )
    .unwrap();
    assert_eq!(identity.issuer, "https://accounts.google.com");
    assert_eq!(identity.subject, "google-subject");
}

#[test]
fn rejects_untrusted_claims_and_key_identifiers() {
    for (field, value) in [
        ("iss", json!("https://attacker.invalid")),
        ("aud", json!("other-client")),
        ("exp", json!(1)),
        ("email_verified", json!(false)),
        ("nonce", json!("different-nonce")),
        ("sub", json!("")),
        ("azp", json!("other-client")),
    ] {
        let mut input = claims();
        input[field] = value;
        assert_eq!(
            verify(&sign(&input, "test-key"), PUBLIC),
            Err(VerificationError::InvalidToken),
            "{field}"
        );
    }
    assert_eq!(
        verify(&sign(&claims(), "wrong-key"), PUBLIC),
        Err(VerificationError::InvalidToken)
    );
    assert_eq!(
        verify("bad.token.value", PUBLIC),
        Err(VerificationError::InvalidToken)
    );
    let mut parts = sign(&claims(), "test-key")
        .split('.')
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let mut forged = claims();
    forged["sub"] = json!("forged");
    parts[1] = sign(&forged, "test-key")
        .split('.')
        .nth(1)
        .unwrap()
        .to_owned();
    assert_eq!(
        verify(&parts.join("."), PUBLIC),
        Err(VerificationError::InvalidToken)
    );
}

#[test]
fn multiple_audiences_require_authorized_party() {
    let mut input = claims();
    input["aud"] = json!(["mobile-client", "other-client"]);
    assert_eq!(
        verify(&sign(&input, "test-key"), PUBLIC),
        Err(VerificationError::InvalidToken)
    );
    input["azp"] = json!("mobile-client");
    assert_eq!(verify(&sign(&input, "test-key"), PUBLIC), Ok(()));
}
