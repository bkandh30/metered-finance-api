use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::finance::{Currency, TransactionType};
use super::keys::Scope;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateAccountRequest {
    #[schema(example = "user_123")]
    pub account_id: String,
    
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

impl CreateAccountRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.account_id.is_empty() {
            return Err("Account ID cannot be empty".to_string());
        }
        
        if self.account_id.len() < 3 {
            return Err("Account ID must be at least 3 characters".to_string());
        }
        
        if self.account_id.len() > 255 {
            return Err("Account ID must not exceed 255 characters".to_string());
        }
        
        if !self.account_id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err("Account ID must contain only alphanumeric characters, underscores, and hyphens".to_string());
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateAccountRequest {
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateTransactionRequest {
    #[schema(example = "user_123")]
    pub account_id: String,
    
    #[schema(example = 99.99)]
    pub amount: f64,
    
    #[schema(example = "USD")]
    pub currency: Currency,
    
    pub transaction_type: TransactionType,
    
    #[serde(default)]
    pub description: Option<String>,
    
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

impl CreateTransactionRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.account_id.is_empty() {
            return Err("Account ID cannot be empty".to_string());
        }
        
        if self.amount <= 0.0 {
            return Err("Amount must be positive".to_string());
        }
        
        if self.amount.is_nan() || self.amount.is_infinite() {
            return Err("Amount must be a valid number".to_string());
        }
        
        let amount_str = format!("{:.2}", self.amount);
        let parsed: f64 = amount_str.parse().unwrap_or(0.0);
        if (parsed - self.amount).abs() > 0.001 {
            return Err("Amount must have at most 2 decimal places".to_string());
        }
        
        if let Some(desc) = &self.description {
            if desc.len() > 1000 {
                return Err("Description must not exceed 1000 characters".to_string());
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateApiKeyRequest {
    #[schema(example = "Production Key")]
    pub name: String,
    
    pub scopes: Vec<Scope>,
    
    #[serde(default)]
    pub rate_limit_per_minute: Option<i32>,
    
    #[serde(default)]
    pub daily_quota: Option<i32>,
    
    #[serde(default)]
    pub monthly_quota: Option<i32>,
}

impl CreateApiKeyRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        
        if self.name.len() < 3 {
            return Err("Name must be at least 3 characters".to_string());
        }
        
        if self.name.len() > 100 {
            return Err("Name must not exceed 100 characters".to_string());
        }
        
        if self.scopes.is_empty() {
            return Err("At least one scope is required".to_string());
        }
        
        if let Some(rate) = self.rate_limit_per_minute {
            if rate < 1 || rate > 10000 {
                return Err("Rate limit must be between 1 and 10000".to_string());
            }
        }
        
        if let Some(daily) = self.daily_quota {
            if daily < 1 || daily > 10_000_000 {
                return Err("Daily quota must be between 1 and 10,000,000".to_string());
            }
        }
        
        if let Some(monthly) = self.monthly_quota {
            if monthly < 1 || monthly > 100_000_000 {
                return Err("Monthly quota must be between 1 and 100,000,000".to_string());
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateApiKeyRequest {
    #[serde(default)]
    pub active: Option<bool>,
    
    #[serde(default)]
    pub scopes: Option<Vec<Scope>>,
    
    #[serde(default)]
    pub rate_limit_per_minute: Option<i32>,
    
    #[serde(default)]
    pub daily_quota: Option<i32>,
    
    #[serde(default)]
    pub monthly_quota: Option<i32>,
}

impl UpdateApiKeyRequest {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(scopes) = &self.scopes {
            if scopes.is_empty() {
                return Err("Scopes cannot be empty if provided".to_string());
            }
        }
        
        if let Some(rate) = self.rate_limit_per_minute {
            if rate < 1 || rate > 10000 {
                return Err("Rate limit must be between 1 and 10000".to_string());
            }
        }
        
        if let Some(daily) = self.daily_quota {
            if daily < 1 || daily > 10_000_000 {
                return Err("Daily quota must be between 1 and 10,000,000".to_string());
            }
        }
        
        if let Some(monthly) = self.monthly_quota {
            if monthly < 1 || monthly > 100_000_000 {
                return Err("Monthly quota must be between 1 and 100,000,000".to_string());
            }
        }
        
        Ok(())
    }
}