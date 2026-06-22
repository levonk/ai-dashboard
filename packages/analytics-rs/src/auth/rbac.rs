use crate::auth::models::{Role, Permission, UserId};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use anyhow::Result;

/// Role-based access control manager
pub struct RoleManager {
    role_assignments: Arc<RwLock<HashMap<UserId, Role>>>,
    custom_permissions: Arc<RwLock<HashMap<UserId, HashSet<Permission>>>>,
}

impl RoleManager {
    /// Create a new role manager
    pub fn new() -> Self {
        Self {
            role_assignments: Arc::new(RwLock::new(HashMap::new())),
            custom_permissions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Assign a role to a user
    pub fn assign_role(&self, user_id: UserId, role: Role) -> Result<()> {
        let mut assignments = self.role_assignments.write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock on role assignments: {}", e))?;
        assignments.insert(user_id, role);
        Ok(())
    }
    
    /// Get a user's role
    pub fn get_role(&self, user_id: &UserId) -> Result<Option<Role>> {
        let assignments = self.role_assignments.read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock on role assignments: {}", e))?;
        Ok(assignments.get(user_id).copied())
    }
    
    /// Remove a user's role assignment
    pub fn remove_role(&self, user_id: &UserId) -> Result<()> {
        let mut assignments = self.role_assignments.write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock on role assignments: {}", e))?;
        assignments.remove(user_id);
        Ok(())
    }
    
    /// Grant custom permission to a user (in addition to role permissions)
    pub fn grant_permission(&self, user_id: UserId, permission: Permission) -> Result<()> {
        let mut custom_perms = self.custom_permissions.write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock on custom permissions: {}", e))?;
        custom_perms.entry(user_id).or_insert_with(HashSet::new).insert(permission);
        Ok(())
    }
    
    /// Revoke custom permission from a user
    pub fn revoke_permission(&self, user_id: &UserId, permission: &Permission) -> Result<()> {
        let mut custom_perms = self.custom_permissions.write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock on custom permissions: {}", e))?;
        if let Some(perms) = custom_perms.get_mut(user_id) {
            perms.remove(permission);
        }
        Ok(())
    }
    
    /// Get all permissions for a user (role + custom)
    pub fn get_user_permissions(&self, user_id: &UserId) -> Result<HashSet<Permission>> {
        let mut permissions = HashSet::new();
        
        // Get role permissions
        if let Some(role) = self.get_role(user_id)? {
            permissions.extend(role.permissions());
        }
        
        // Get custom permissions
        let custom_perms = self.custom_permissions.read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock on custom permissions: {}", e))?;
        if let Some(perms) = custom_perms.get(user_id) {
            permissions.extend(perms.clone());
        }
        
        Ok(permissions)
    }
}

impl Default for RoleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Permission checker for authorization decisions
pub struct PermissionChecker {
    role_manager: RoleManager,
}

impl PermissionChecker {
    /// Create a new permission checker
    pub fn new(role_manager: RoleManager) -> Self {
        Self { role_manager }
    }
    
    /// Check if a user has a specific permission
    pub fn has_permission(&self, user_id: &UserId, permission: &Permission) -> Result<bool> {
        let permissions = self.role_manager.get_user_permissions(user_id)?;
        Ok(permissions.contains(permission))
    }
    
    /// Check if a user has all specified permissions
    pub fn has_all_permissions(&self, user_id: &UserId, permissions: &[Permission]) -> Result<bool> {
        let user_permissions = self.role_manager.get_user_permissions(user_id)?;
        Ok(permissions.iter().all(|p| user_permissions.contains(p)))
    }
    
    /// Check if a user has any of the specified permissions
    pub fn has_any_permission(&self, user_id: &UserId, permissions: &[Permission]) -> Result<bool> {
        let user_permissions = self.role_manager.get_user_permissions(user_id)?;
        Ok(permissions.iter().any(|p| user_permissions.contains(p)))
    }
    
    /// Check if a user has a specific role
    pub fn has_role(&self, user_id: &UserId, role: Role) -> Result<bool> {
        Ok(self.role_manager.get_role(user_id)? == Some(role))
    }
    
    /// Check if a user is an admin
    pub fn is_admin(&self, user_id: &UserId) -> Result<bool> {
        self.has_role(user_id, Role::Admin)
    }
}

/// Access control for authorization enforcement
pub struct AccessControl {
    permission_checker: PermissionChecker,
}

impl AccessControl {
    /// Create a new access control instance
    pub fn new(permission_checker: PermissionChecker) -> Self {
        Self { permission_checker }
    }
    
    /// Authorize a user for a specific permission
    /// Returns Ok(()) if authorized, Err otherwise
    pub fn authorize(&self, user_id: &UserId, permission: Permission) -> Result<()> {
        if self.permission_checker.has_permission(user_id, &permission)? {
            Ok(())
        } else {
            anyhow::bail!("User {:?} does not have permission {:?}", user_id, permission);
        }
    }
    
    /// Authorize a user for multiple permissions (all required)
    pub fn authorize_all(&self, user_id: &UserId, permissions: &[Permission]) -> Result<()> {
        if self.permission_checker.has_all_permissions(user_id, permissions)? {
            Ok(())
        } else {
            anyhow::bail!("User {:?} does not have all required permissions", user_id);
        }
    }
    
    /// Authorize a user for multiple permissions (any required)
    pub fn authorize_any(&self, user_id: &UserId, permissions: &[Permission]) -> Result<()> {
        if self.permission_checker.has_any_permission(user_id, permissions)? {
            Ok(())
        } else {
            anyhow::bail!("User {:?} does not have any of the required permissions", user_id);
        }
    }
    
    /// Authorize admin access only
    pub fn authorize_admin(&self, user_id: &UserId) -> Result<()> {
        if self.permission_checker.is_admin(user_id)? {
            Ok(())
        } else {
            anyhow::bail!("Admin access required for user {:?}", user_id);
        }
    }
    
    /// Check authorization without returning error
    pub fn check_authorization(&self, user_id: &UserId, permission: Permission) -> bool {
        self.permission_checker.has_permission(user_id, &permission).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_role_assignment() {
        let manager = RoleManager::new();
        let user_id = UserId::new_v4();
        
        manager.assign_role(user_id, Role::Admin).unwrap();
        let role = manager.get_role(&user_id).unwrap();
        
        assert_eq!(role, Some(Role::Admin));
    }
    
    #[test]
    fn test_permission_checking() {
        let role_manager = RoleManager::new();
        let user_id = UserId::new_v4();
        
        role_manager.assign_role(user_id, Role::Viewer).unwrap();
        let checker = PermissionChecker::new(role_manager);
        
        // Viewer should have ReadAnalytics permission
        assert!(checker.has_permission(&user_id, &Permission::ReadAnalytics).unwrap());
        
        // Viewer should not have DeleteAnalytics permission
        assert!(!checker.has_permission(&user_id, &Permission::DeleteAnalytics).unwrap());
    }
    
    #[test]
    fn test_custom_permissions() {
        let role_manager = RoleManager::new();
        let user_id = UserId::new_v4();
        
        role_manager.assign_role(user_id, Role::Viewer).unwrap();
        role_manager.grant_permission(user_id, Permission::DeleteAnalytics).unwrap();
        
        let checker = PermissionChecker::new(role_manager);
        
        // Viewer with custom permission should have DeleteAnalytics
        assert!(checker.has_permission(&user_id, &Permission::DeleteAnalytics).unwrap());
    }
    
    #[test]
    fn test_access_control() {
        let role_manager = RoleManager::new();
        let user_id = UserId::new_v4();
        
        role_manager.assign_role(user_id, Role::Viewer).unwrap();
        let checker = PermissionChecker::new(role_manager);
        let access_control = AccessControl::new(checker);
        
        // Should succeed
        assert!(access_control.authorize(&user_id, Permission::ReadAnalytics).is_ok());
        
        // Should fail
        assert!(access_control.authorize(&user_id, Permission::DeleteAnalytics).is_err());
    }
}