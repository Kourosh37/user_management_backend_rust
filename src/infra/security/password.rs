use crate::domain::DomainError;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use password_hash::SaltString;
use rand::rngs::OsRng;

pub fn hash_password(plain: &str) -> Result<String, DomainError> {
    let salt = SaltString::generate(&mut OsRng);
    let params = Params::new(19_456, 2, 1, None)
        .map_err(|err| DomainError::Internal(err.to_string()))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let hash = argon2
        .hash_password(plain.as_bytes(), &salt)
        .map_err(|err| DomainError::Internal(err.to_string()))?
        .to_string();
    Ok(hash)
}

pub fn verify_password(hash: &str, candidate: &str) -> Result<bool, DomainError> {
    let parsed = PasswordHash::new(hash).map_err(|err| DomainError::Internal(err.to_string()))?;
    Ok(Argon2::default()
        .verify_password(candidate.as_bytes(), &parsed)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_round_trip() {
        let hash = hash_password("p@ssword").unwrap();
        assert!(verify_password(&hash, "p@ssword").unwrap());
        assert!(!verify_password(&hash, "wrong").unwrap());
    }
}
