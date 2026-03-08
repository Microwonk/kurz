use axum_extra::extract::CookieJar;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::{ApiError, ApiResult, AppState};

pub const SESSION_COOKIE: &str = "kurz_session";

pub fn authentication_gate(jar: &CookieJar, state: &AppState) -> ApiResult<()> {
    if cfg!(feature = "disable_auth") {
        Ok(())
    } else {
        jar.get(SESSION_COOKIE)
            .map(|c| state.sessions.is_valid(c.value()).is_ok_and(|r| r))
            .unwrap_or(false)
            .ok_or(ApiError::unauthorized())
    }
}

#[derive(Clone)]
pub struct SessionStore {
    tokens: Arc<Mutex<HashSet<String>>>,
    password_hash: String,
}

impl SessionStore {
    pub fn new(password: &str) -> Self {
        Self {
            tokens: Arc::new(Mutex::new(HashSet::new())),
            password_hash: hash(password),
        }
    }

    pub fn login(&self, password: &str) -> Option<String> {
        if hash(password) == self.password_hash {
            let token = Uuid::new_v4().to_string();
            self.tokens.lock().ok()?.insert(token.clone());
            Some(token)
        } else {
            None
        }
    }

    pub fn is_valid(&self, token: &str) -> ApiResult<bool> {
        Ok(self
            .tokens
            .lock()
            .map_err(ApiError::internal_server_error)?
            .contains(token))
    }

    pub fn logout(&self, token: &str) -> ApiResult<bool> {
        Ok(self
            .tokens
            .lock()
            .map_err(ApiError::internal_server_error)?
            .remove(token))
    }
}

fn hash(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hex::encode(hasher.finalize())
}
