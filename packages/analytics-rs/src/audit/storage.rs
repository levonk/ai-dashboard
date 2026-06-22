use crate::audit::models::{AuditEvent, AuditQuery};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// In-memory audit storage (for development - replace with persistent storage in production)
pub struct InMemoryAuditStorage {
    events: Arc<RwLock<Vec<AuditEvent>>>,
    user_index: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
    session_index: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
}

impl InMemoryAuditStorage {
    /// Create a new in-memory audit storage
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            user_index: Arc::new(RwLock::new(HashMap::new())),
            session_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Store an audit event
    pub fn store_event(&self, event: AuditEvent) -> Result<()> {
        let event_id = event.id;
        let user_id = event.user_id;
        let session_id = event.session_id;
        
        // Store event
        {
            let mut events = self.events.write()
                .map_err(|e| anyhow::anyhow!("Failed to acquire write lock on events: {}", e))?;
            events.push(event);
        }
        
        // Update user index
        if let Some(user_id) = user_id {
            let mut user_index = self.user_index.write()
                .map_err(|e| anyhow::anyhow!("Failed to acquire write lock on user index: {}", e))?;
            user_index.entry(user_id).or_insert_with(Vec::new).push(event_id);
        }
        
        // Update session index
        if let Some(session_id) = session_id {
            let mut session_index = self.session_index.write()
                .map_err(|e| anyhow::anyhow!("Failed to acquire write lock on session index: {}", e))?;
            session_index.entry(session_id).or_insert_with(Vec::new).push(event_id);
        }
        
        Ok(())
    }
    
    /// Query audit events
    pub fn query_events(&self, query: &AuditQuery) -> Result<Vec<AuditEvent>> {
        let events = self.events.read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock on events: {}", e))?;
        
        let mut filtered: Vec<AuditEvent> = events.iter().cloned().collect();
        
        // Apply filters
        if let Some(event_types) = &query.filters.event_types {
            filtered.retain(|e| event_types.contains(&e.event_type));
        }
        
        if let Some(user_id) = &query.filters.user_id {
            filtered.retain(|e| e.user_id.as_ref() == Some(user_id));
        }
        
        if let Some(session_id) = &query.filters.session_id {
            filtered.retain(|e| e.session_id.as_ref() == Some(session_id));
        }
        
        if let Some(severity) = &query.filters.severity {
            filtered.retain(|e| &e.severity == severity);
        }
        
        if let Some(success) = &query.filters.success {
            filtered.retain(|e| e.success == *success);
        }
        
        if let Some(start_time) = &query.filters.start_time {
            filtered.retain(|e| e.timestamp >= *start_time);
        }
        
        if let Some(end_time) = &query.filters.end_time {
            filtered.retain(|e| e.timestamp <= *end_time);
        }
        
        if let Some(resource) = &query.filters.resource {
            filtered.retain(|e| e.resource.contains(resource));
        }
        
        if let Some(action) = &query.filters.action {
            filtered.retain(|e| e.action.contains(action));
        }
        
        // Sort
        if let Some(order_by) = &query.order_by {
            let reverse = query.order_direction.as_deref() == Some("desc");
            match order_by.as_str() {
                "timestamp" => {
                    filtered.sort_by(|a, b| {
                        if reverse {
                            b.timestamp.cmp(&a.timestamp)
                        } else {
                            a.timestamp.cmp(&b.timestamp)
                        }
                    });
                }
                "severity" => {
                    filtered.sort_by(|a, b| {
                        if reverse {
                            b.severity.cmp(&a.severity)
                        } else {
                            a.severity.cmp(&b.severity)
                        }
                    });
                }
                _ => {}
            }
        }
        
        // Apply limit and offset
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(100);
        
        let total = filtered.len();
        let end = std::cmp::min(offset + limit, total);
        
        if offset < total {
            Ok(filtered[offset..end].to_vec())
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Get events by user ID
    pub fn get_events_by_user(&self, user_id: &Uuid) -> Result<Vec<AuditEvent>> {
        let event_ids = {
            let user_index = self.user_index.read()
                .map_err(|e| anyhow::anyhow!("Failed to acquire read lock on user index: {}", e))?;
            user_index.get(user_id).cloned().unwrap_or_default()
        };
        
        let events = self.events.read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock on events: {}", e))?;
        
        let mut result = Vec::new();
        for event_id in event_ids {
            if let Some(event) = events.iter().find(|e| e.id == event_id) {
                result.push(event.clone());
            }
        }
        
        Ok(result)
    }
    
    /// Get events by session ID
    pub fn get_events_by_session(&self, session_id: &Uuid) -> Result<Vec<AuditEvent>> {
        let event_ids = {
            let session_index = self.session_index.read()
                .map_err(|e| anyhow::anyhow!("Failed to acquire read lock on session index: {}", e))?;
            session_index.get(session_id).cloned().unwrap_or_default()
        };
        
        let events = self.events.read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock on events: {}", e))?;
        
        let mut result = Vec::new();
        for event_id in event_ids {
            if let Some(event) = events.iter().find(|e| e.id == event_id) {
                result.push(event.clone());
            }
        }
        
        Ok(result)
    }
    
    /// Clean up old events
    pub fn cleanup_old_events(&self, older_than: chrono::Duration) -> Result<usize> {
        let cutoff = chrono::Utc::now() - older_than;
        
        let events_to_remove: Vec<Uuid> = {
            let events = self.events.read()
                .map_err(|e| anyhow::anyhow!("Failed to acquire read lock on events: {}", e))?;
            events.iter()
                .filter(|e| e.timestamp < cutoff)
                .map(|e| e.id)
                .collect()
        };
        
        let count = events_to_remove.len();
        
        for event_id in events_to_remove {
            self.remove_event(&event_id)?;
        }
        
        Ok(count)
    }
    
    /// Remove an event
    fn remove_event(&self, event_id: &Uuid) -> Result<()> {
        let user_id = {
            let events = self.events.read()
                .map_err(|e| anyhow::anyhow!("Failed to acquire read lock on events: {}", e))?;
            events.iter()
                .find(|e| e.id == *event_id)
                .and_then(|e| e.user_id)
        };
        
        let session_id = {
            let events = self.events.read()
                .map_err(|e| anyhow::anyhow!("Failed to acquire read lock on events: {}", e))?;
            events.iter()
                .find(|e| e.id == *event_id)
                .and_then(|e| e.session_id)
        };
        
        // Remove from main storage
        {
            let mut events = self.events.write()
                .map_err(|e| anyhow::anyhow!("Failed to acquire write lock on events: {}", e))?;
            events.retain(|e| e.id != *event_id);
        }
        
        // Remove from user index
        if let Some(user_id) = user_id {
            let mut user_index = self.user_index.write()
                .map_err(|e| anyhow::anyhow!("Failed to acquire write lock on user index: {}", e))?;
            if let Some(event_ids) = user_index.get_mut(&user_id) {
                event_ids.retain(|id| id != event_id);
            }
        }
        
        // Remove from session index
        if let Some(session_id) = session_id {
            let mut session_index = self.session_index.write()
                .map_err(|e| anyhow::anyhow!("Failed to acquire write lock on session index: {}", e))?;
            if let Some(event_ids) = session_index.get_mut(&session_id) {
                event_ids.retain(|id| id != event_id);
            }
        }
        
        Ok(())
    }
}

impl Default for InMemoryAuditStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for audit storage implementations
pub trait AuditStorage: Send + Sync {
    fn store_event(&self, event: AuditEvent) -> Result<()>;
    fn query_events(&self, query: &AuditQuery) -> Result<Vec<AuditEvent>>;
    fn get_events_by_user(&self, user_id: &Uuid) -> Result<Vec<AuditEvent>>;
    fn get_events_by_session(&self, session_id: &Uuid) -> Result<Vec<AuditEvent>>;
    fn cleanup_old_events(&self, older_than: chrono::Duration) -> Result<usize>;
}

impl AuditStorage for InMemoryAuditStorage {
    fn store_event(&self, event: AuditEvent) -> Result<()> {
        self.store_event(event)
    }
    
    fn query_events(&self, query: &AuditQuery) -> Result<Vec<AuditEvent>> {
        self.query_events(query)
    }
    
    fn get_events_by_user(&self, user_id: &Uuid) -> Result<Vec<AuditEvent>> {
        self.get_events_by_user(user_id)
    }
    
    fn get_events_by_session(&self, session_id: &Uuid) -> Result<Vec<AuditEvent>> {
        self.get_events_by_session(session_id)
    }
    
    fn cleanup_old_events(&self, older_than: chrono::Duration) -> Result<usize> {
        self.cleanup_old_events(older_than)
    }
}