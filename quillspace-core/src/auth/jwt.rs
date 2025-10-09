use josekit::{
    jwt::{JwtPayload},
    jws::{HS256, JwsHeader},
    JoseError,
};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};
use anyhow::anyhow;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // Subject (user ID)
    pub email: String,      // User email
    pub name: String,       // User name
    pub role: String,       // User role
    pub tenant_id: String,  // Tenant ID
    pub exp: i64,          // Expiration time
    pub iat: i64,          // Issued at
    pub iss: String,       // Issuer
}

pub struct JwtManager {
    secret: Vec<u8>,
    issuer: String,
}

impl JwtManager {
    pub fn new(secret: &str, issuer: &str) -> Self {
        Self {
            secret: secret.as_bytes().to_vec(),
            issuer: issuer.to_string(),
        }
    }

    pub fn generate_token(&self, user_id: &str, email: &str, name: &str, role: &str, tenant_id: &str) -> Result<String, JoseError> {
        let now = Utc::now();
        let exp = now + Duration::hours(24 * 7); // 7 days

        let mut payload = JwtPayload::new();
        payload.set_subject(user_id);
        payload.set_claim("email", Some(serde_json::Value::String(email.to_string())))?;
        payload.set_claim("name", Some(serde_json::Value::String(name.to_string())))?;
        payload.set_claim("role", Some(serde_json::Value::String(role.to_string())))?;
        payload.set_claim("tenant_id", Some(serde_json::Value::String(tenant_id.to_string())))?;
        
        // Convert chrono DateTime to SystemTime
        let exp_system_time = UNIX_EPOCH + std::time::Duration::from_secs(exp.timestamp() as u64);
        let now_system_time = UNIX_EPOCH + std::time::Duration::from_secs(now.timestamp() as u64);
        
        payload.set_expires_at(&exp_system_time);
        payload.set_issued_at(&now_system_time);
        payload.set_issuer(&self.issuer);

        let header = JwsHeader::new();
        let signer = HS256.signer_from_bytes(&self.secret)?;
        let token = josekit::jwt::encode_with_signer(&payload, &header, &signer)?;

        Ok(token)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, JoseError> {
        let verifier = HS256.verifier_from_bytes(&self.secret)?;
        let (payload, _header) = josekit::jwt::decode_with_verifier(token, &verifier)?;

        let sub = payload.subject()
            .ok_or_else(|| JoseError::InvalidJwtFormat(anyhow!("Missing subject")))?;
        
        let email = payload.claim("email")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JoseError::InvalidJwtFormat(anyhow!("Missing email")))?;
        
        let name = payload.claim("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JoseError::InvalidJwtFormat(anyhow!("Missing name")))?;
        
        let role = payload.claim("role")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JoseError::InvalidJwtFormat(anyhow!("Missing role")))?;
        
        let tenant_id = payload.claim("tenant_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JoseError::InvalidJwtFormat(anyhow!("Missing tenant_id")))?;

        let exp = payload.expires_at()
            .ok_or_else(|| JoseError::InvalidJwtFormat(anyhow!("Missing expiration")))?
            .duration_since(UNIX_EPOCH)
            .map_err(|_| JoseError::InvalidJwtFormat(anyhow!("Invalid expiration time")))?
            .as_secs() as i64;
        
        let iat = payload.issued_at()
            .ok_or_else(|| JoseError::InvalidJwtFormat(anyhow!("Missing issued_at")))?
            .duration_since(UNIX_EPOCH)
            .map_err(|_| JoseError::InvalidJwtFormat(anyhow!("Invalid issued_at time")))?
            .as_secs() as i64;

        let iss = payload.issuer()
            .ok_or_else(|| JoseError::InvalidJwtFormat(anyhow!("Missing issuer")))?;

        Ok(Claims {
            sub: sub.to_string(),
            email: email.to_string(),
            name: name.to_string(),
            role: role.to_string(),
            tenant_id: tenant_id.to_string(),
            exp,
            iat,
            iss: iss.to_string(),
        })
    }

    pub fn is_token_valid(&self, token: &str) -> bool {
        match self.verify_token(token) {
            Ok(claims) => {
                let now = Utc::now().timestamp();
                claims.exp > now
            },
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_generation_and_verification() {
        let jwt_manager = JwtManager::new("test-secret-key", "quillspace");
        
        let token = jwt_manager.create_token(
            "user-123",
            "test@example.com",
            "Test User",
            "admin",
            "tenant-456"
        ).expect("Failed to create test token");

        let claims = jwt_manager.verify_token(&token).expect("Failed to verify test token");
        
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.name, "Test User");
        assert_eq!(claims.role, "admin");
        assert_eq!(claims.tenant_id, "tenant-456");
        assert_eq!(claims.iss, "quillspace");
    }

    #[test]
    fn test_token_validation() {
        let jwt_manager = JwtManager::new("test-secret-key", "quillspace");
        
        let token = jwt_manager.create_token(
            "user-123",
            "test@example.com",
            "Test User",
            "admin",
            "tenant-456"
        ).expect("Failed to create test token");

        assert!(jwt_manager.is_token_valid(&token));
        assert!(!jwt_manager.is_token_valid("invalid-token"));
    }
}
