use crate::auth::models::{Session, SessionId, UserId, AuthError, AuthToken, TokenType};
use anyhow::Result;
use chrono::{Utc, Duration};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// In-memory session store (for development - replace with persistent store in production)
pub struct SessionStore {
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    user_sessions: Arc<RwLock<HashMap<UserId, Vec<SessionId>>>>,
}

impl SessionStore {
    /// Create a new in-memory session store
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            user_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Store a session
    pub fn store_session(&self, session: Session) -> Result<()> {
        let session_id = session.id;
        let user_id = session.user_id;
        
        // Store session
        {
            let mut sessions = self.sessions.write()
                .map_err(|e| anyhow::anyhow!("Failed to acquire write lock on sessions: {}", e))?;
            sessions.insert(session_id, session);
        }
        
        // Update user session index
        {
            let mut user_sessions = self.user_sessions.write()
                .map_err(|e| anyhow::anyhow!("Failed to acquire write lock on user sessions: {}", e))?;
            user_sessions.entry(user_id).or_insert_with(Vec::new).push(session_id);
        }
        
        Ok(())
    }
    
    /// Retrieve a session by ID
    pub fn get_session(&self, session_id: &SessionId) -> Result<Option<Session>> {
        let sessions = self.sessions.read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock on sessions: {}", e))?;
        Ok(sessions.get(session_id).cloned())
    }
    
    /// Remove a session
    pub fn remove_session(&self, session_id: &SessionId) -> Result<()> {
        let user_id = {
            let mut sessions = self.sessions.write()
                .map_err(|e| anyhow::anyhow!("Failed to acquire write lock on sessions: {}", e))?;
            sessions.remove(session_id).map(|s| s.user_id)
        };
        
        if let Some(user_id) = user_id {
            let mut user_sessions = self.user_sessions.write()
                .map_err(|e| anyhow::anyhow!("Failed to acquire write lock on user sessions: {}", e))?;
            if let Some(session_ids) = user_sessions.get_mut(&user_id) {
                session_ids.retain(|id| id != session_id);
            }
        }
        
        Ok(())
    }
    
    /// Get all sessions for a user
    pub fn get_user_sessions(&self, user_id: &UserId) -> Result<Vec<Session>> {
        let session_ids = {
            let user_sessions = self.user_sessions.read()
                .map_err(|e| anyhow::anyhow!("Failed to acquire read lock on user sessions: {}", e))?;
            user_sessions.get(user_id).cloned().unwrap_or_default()
        };
        
        let sessions = self.sessions.read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock on sessions: {}", e))?;
        
        let mut result = Vec::new();
        for session_id in session_ids {
            if let Some(session) = sessions.get(&session_id) {
                result.push(session.clone());
            }
        }
        
        Ok(result)
    }
    
    /// Remove all sessions for a user
    pub fn remove_user_sessions(&self, user_id: &UserId) -> Result<()> {
        let session_ids = {
            let mut user_sessions = self.user_sessions.write()
                .map_err(|e| anyhow::anyhow!("Failed to acquire write lock on user sessions: {}", e))?;
            user_sessions.remove(user_id).unwrap_or_default()
        };
        
        let mut sessions = self.sessions.write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock on sessions: {}", e))?;
        
        for session_id in session_ids {
            sessions.remove(&session_id);
        }
        
        Ok(())
    }
    
    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&self) -> Result<usize> {
        let now = Utc::now();
        let expired_ids: Vec<SessionId> = {
            let sessions = self.sessions.read()
                .map_err(|e| anyhow::anyhow!("Failed to acquire read lock on sessions: {}", e))?;
            sessions.iter()
                .filter(|(_, session)| session.expires_at < now)
                .map(|(id, _)| *id)
                .collect()
        };
        
        let count = expired_ids.len();
        for session_id in &expired_ids {
            self.remove_session(session_id)?;
        }
        
        Ok(count)
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Session manager for handling user sessions
pub struct SessionManager {
    store: SessionStore,
    session_duration_hours: i64,
    max_sessions_per_user: usize,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            store: SessionStore::new(),
            session_duration_hours: 24,
            max_sessions_per_user: 5,
        }
    }
    
    /// Create with custom session duration
    pub fn with_duration(hours: i64) -> Self {
        let mut manager = Self::new();
        manager.session_duration_hours = hours;
        manager
    }
    
    /// Create a new session for a user
    pub fn create_session(
        &self,
        user_id: UserId,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<Session> {
        // Check max sessions per user
        let user_sessions = self.store.get_user_sessions(&user_id)?;
        if user_sessions.len() >= self.max_sessions_per_user {
            // Remove oldest session
            if let Some(oldest) = user_sessions.iter().min_by_key(|s| s.created_at) {
                self.store.remove_session(&oldest.id)?;
            }
        }
        
        let now = Utc::now();
        let expires_at = now + Duration::hours(self.session_duration_hours);
        
        let session = Session {
            id: SessionId::new_v4(),
            user_id,
            token: self.generate_token(),
            created_at: now,
            expires_at,
            last_activity: now,
            ip_address,
            user_agent,
        };
        
        self.store.store_session(session.clone())?;
        Ok(session)
    }
    
    /// Validate a session token
    pub fn validate_session(&self, token: &str) -> Result<Session> {
        // Find session by token (linear search - optimize in production)
        let session_id = {
            let sessions = self.store.sessions.read()
                .map_err(|e| anyhow::anyhow!("Failed to acquire read lock on sessions: {}", e))?;
            
            let found_session = sessions.values()
                .find(|session| session.token == token)
                .ok_or(AuthError::SessionNotFound)?;
            
            // Check if expired
            if found_session.expires_at < Utc::now() {
                return Err(AuthError::SessionExpired.into());
            }
            
            found_session.id
        };
        
        // Update last activity
        self.update_activity(&session_id)?;
        
        // Get the session again to return it
        self.store.get_session(&session_id)?
            .ok_or(AuthError::SessionNotFound.into())
    }
    
    /// Refresh a session (extend expiration)
    pub fn refresh_session(&self, session_id: &SessionId) -> Result<Session> {
        let mut session = self.store.get_session(session_id)?
            .ok_or(AuthError::SessionNotFound)?;
        
        // Check if expired
        if session.expires_at < Utc::now() {
            return Err(AuthError::SessionExpired.into());
        }
        
        // Extend expiration
        session.expires_at = Utc::now() + Duration::hours(self.session_duration_hours);
        session.last_activity = Utc::now();
        
        self.store.store_session(session.clone())?;
        Ok(session)
    }
    
    /// Revoke a session
    pub fn revoke_session(&self, session_id: &SessionId) -> Result<()> {
        self.store.remove_session(session_id)
    }
    
    /// Revoke all sessions for a user
    pub fn revoke_user_sessions(&self, user_id: &UserId) -> Result<()> {
        self.store.remove_user_sessions(user_id)
    }
    
    /// Update session activity timestamp
    fn update_activity(&self, session_id: &SessionId) -> Result<()> {
        if let Some(mut session) = self.store.get_session(session_id)? {
            session.last_activity = Utc::now();
            self.store.store_session(session)?;
        }
        Ok(())
    }
    
    /// Generate a secure random token
    fn generate_token(&self) -> String {
        use rand::Rng;
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill(&mut bytes);
        hex::encode(bytes)
    }
    
    /// Get session as auth token
    pub fn as_auth_token(&self, session: &Session) -> AuthToken {
        AuthToken {
            token: session.token.clone(),
            token_type: TokenType::Session,
            expires_at: session.expires_at,
        }
    }
    
    /// Clean up expired sessions
    pub fn cleanup(&self) -> Result<usize> {
        self.store.cleanup_expired_sessions()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_session_creation() {
        let manager = SessionManager::new();
        let user_id = UserId::new_v4();
        
        let session = manager.create_session(user_id, Some("127.0.0.1".to_string()), None).unwrap();
        
        assert_eq!(session.user_id, user_id);
        assert!(!session.token.is_empty());
        assert!(session.expires_at > Utc::now());
    }
    
    #[test]
    fn test_session_validation() {
        let manager = SessionManager::new();
        let user_id = UserId::new_v4();
        
        let session = manager.create_session(user_id, None, None).unwrap();
        let validated = manager.validate_session(&session.token).unwrap();
        
        assert_eq!(validated.id, session.id);
    }
    
    #[test]
    fn test_session_revocation() {
        let manager = SessionManager::new();
        let user_id = UserId::new_v4();
        
        let session = manager.create_session(user_id, None, None).unwrap();
        manager.revoke_session(&session.id).unwrap();
        
        let result = manager.validate_session(&session.token);
        assert!(result.is_err());
    }
}