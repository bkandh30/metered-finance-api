use serde::{Deserialize, Serialize};
use sqlx::Type;
use utoipa::ToSchema;
use time::OffsetDateTime;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

#[derive(Debug, Clone, Serialize, Deserialize, Type, ToSchema, PartialEq, Eq)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    Client,
    Admin,
    Reporting,
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scope::Client => write!(f, "client"),
            Scope::Admin => write!(f, "admin"),
            Scope::Reporting => write!(f, "reporting"),
        }
    }
}

impl Scope {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "client" => Some(Scope::Client),
            "admin" => Some(Scope::Admin),
            "reporting" => Some(Scope::Reporting),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiKey {
    pub key_id: String,
    pub prefix: String,
    #[serde(skip_serializing)]
    pub secret_hash: String,
    pub scopes: Vec<Scope>,
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_used_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateApiKeyRequest {
    pub scopes: Vec<String>,
}

impl CreateApiKeyRequest {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.scopes.is_empty() {
            return Err(ValidationError::EmptyScopes);
        }
        
        let mut unique_scopes = self.scopes.clone();
        unique_scopes.sort_by_key(|s| format!("{:?}", s));
        unique_scopes.dedup();
        
        if unique_scopes.len() != self.scopes.len() {
            return Err(ValidationError::DuplicateScopes);
        }
        
        for scope in &unique_scopes {
            if !Scope::from_str(scope).is_some() {
                return Err(ValidationError::InvalidScope);
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateApiKeyResponse {
    pub key_id: String,
    pub api_key: String,
    pub prefix: String,
    pub scopes: Vec<Scope>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct AdminKeyConfig {
    pub secret_hash: String,
}

#[derive(Debug, Clone)]
pub enum AuthContext {
    Client { key_id: String, scopes: Vec<Scope> },
    Admin,
}

impl AuthContext {
    pub fn has_scope(&self, scope: &Scope) -> bool {
        match self {
            AuthContext::Client { scopes, .. } => scopes.contains(scope),
            AuthContext::Admin => true,
        }
    }
    
    pub fn is_admin(&self) -> bool {
        matches!(self, AuthContext::Admin)
    }
    
    pub fn key_id(&self) -> Option<&str> {
        match self {
            AuthContext::Client { key_id, .. } => Some(key_id),
            AuthContext::Admin => None,
        }
    }
}

pub struct ApiKeyGenerator;

impl ApiKeyGenerator {
    pub fn generate(prefix: &str) -> (String, String) {
        use rand::Rng;
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        const KEY_LENGTH: usize = 32;
        
        let mut rng = rand::rng();
        let random_part: String = (0..KEY_LENGTH)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        
        let full_key = format!("{}_{}", prefix, random_part);
        (full_key, prefix.to_string())
    }
    
    pub fn hash_secret(secret: &str) -> Result<String, KeyError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(secret.as_bytes(), &salt)
            .map_err(|_| KeyError::HashingFailed)?;
        
        Ok(password_hash.to_string())
    }
    
    pub fn verify_secret(secret: &str, hash: &str) -> bool {
        use argon2::{PasswordHash, PasswordVerifier};
        
        let parsed_hash = match PasswordHash::new(hash) {
            Ok(h) => h,
            Err(_) => return false,
        };
        
        Argon2::default()
            .verify_password(secret.as_bytes(), &parsed_hash)
            .is_ok()
    }
    
    pub fn extract_prefix(api_key: &str) -> Option<String> {
        api_key.split('_').take(2).collect::<Vec<_>>().get(..2).map(|parts| {
            format!("{}_{}", parts[0], parts[1])
        })
    }
}

pub fn generate_key_id() -> String {
    format!("key_{}", uuid::Uuid::new_v4())
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Scopes cannot be empty")]
    EmptyScopes,
    
    #[error("Duplicate scopes not allowed")]
    DuplicateScopes,
}

#[derive(Debug, thiserror::Error)]
pub enum KeyError {
    #[error("Failed to hash API key")]
    HashingFailed,
    
    #[error("Invalid API key format")]
    InvalidFormat,
    
    #[error("API key not found")]
    NotFound,
    
    #[error("API key is inactive")]
    Inactive,
}