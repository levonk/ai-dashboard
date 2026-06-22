use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use analytics_rs::auth::{User, Role, BasicAuth, AuthResult, AuthError, Credentials};
use std::sync::Arc;

/// User management state
#[derive(Clone)]
pub struct UserState {
    // In production, this would be a database connection
    users: Arc<tokio::sync::RwLock<Vec<User>>>,
    auth: Arc<BasicAuth>,
}

impl UserState {
    pub fn new() -> Self {
        Self {
            users: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            auth: Arc::new(BasicAuth::new()),
        }
    }
}

impl Default for UserState {
    fn default() -> Self {
        Self::new()
    }
}

/// Create user request
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: Role,
}

/// Update user request
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub role: Option<Role>,
    pub password: Option<String>,
}

/// User response
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub mfa_enabled: bool,
}

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// Create a new user
pub async fn create_user(
    State(state): State<UserState>,
    Json(req): Json<CreateUserRequest>,
) -> impl IntoResponse {
    // Validate password strength
    if let Err(e) = state.auth.validate_password_strength(&req.password) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_password".to_string(),
                message: e.to_string(),
            }),
        );
    }

    // Check if username already exists
    {
        let users = state.users.read().await;
        if users.iter().any(|u| u.username == req.username) {
            return (
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    error: "user_exists".to_string(),
                    message: "Username already exists".to_string(),
                }),
            );
        }
    }

    // Hash password
    let password_hash = match state.auth.create_password_hash(&req.password) {
        Ok(hash) => hash,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "hash_error".to_string(),
                    message: e.to_string(),
                }),
            );
        }
    };

    // Create user
    let user = User {
        id: Uuid::new_v4(),
        username: req.username.clone(),
        email: req.email,
        password_hash,
        role: req.role,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login: None,
        failed_login_attempts: 0,
        locked_until: None,
        mfa_enabled: false,
        mfa_secret: None,
    };

    // Store user
    {
        let mut users = state.users.write().await;
        users.push(user.clone());
    }

    let response = UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        role: user.role,
        created_at: user.created_at,
        updated_at: user.updated_at,
        last_login: user.last_login,
        mfa_enabled: user.mfa_enabled,
    };

    (StatusCode::CREATED, Json(response))
}

/// Get user by ID
pub async fn get_user(
    State(state): State<UserState>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    let users = state.users.read().await;
    
    match users.iter().find(|u| u.id == user_id) {
        Some(user) => {
            let response = UserResponse {
                id: user.id,
                username: user.username.clone(),
                email: user.email.clone(),
                role: user.role,
                created_at: user.created_at,
                updated_at: user.updated_at,
                last_login: user.last_login,
                mfa_enabled: user.mfa_enabled,
            };
            (StatusCode::OK, Json(response))
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "user_not_found".to_string(),
                message: "User not found".to_string(),
            }),
        ),
    }
}

/// List all users
pub async fn list_users(State(state): State<UserState>) -> impl IntoResponse {
    let users = state.users.read().await;
    
    let responses: Vec<UserResponse> = users
        .iter()
        .map(|user| UserResponse {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            role: user.role,
            created_at: user.created_at,
            updated_at: user.updated_at,
            last_login: user.last_login,
            mfa_enabled: user.mfa_enabled,
        })
        .collect();

    (StatusCode::OK, Json(responses))
}

/// Update user
pub async fn update_user(
    State(state): State<UserState>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    let mut users = state.users.write().await;
    
    match users.iter_mut().find(|u| u.id == user_id) {
        Some(user) => {
            // Update fields
            if let Some(email) = req.email {
                user.email = email;
            }
            if let Some(role) = req.role {
                user.role = role;
            }
            if let Some(password) = req.password {
                // Validate password strength
                if let Err(e) = state.auth.validate_password_strength(&password) {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: "invalid_password".to_string(),
                            message: e.to_string(),
                        }),
                    );
                }
                
                // Hash new password
                match state.auth.create_password_hash(&password) {
                    Ok(hash) => user.password_hash = hash,
                    Err(e) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: "hash_error".to_string(),
                                message: e.to_string(),
                            }),
                        );
                    }
                }
            }
            
            user.updated_at = Utc::now();
            
            let response = UserResponse {
                id: user.id,
                username: user.username.clone(),
                email: user.email.clone(),
                role: user.role,
                created_at: user.created_at,
                updated_at: user.updated_at,
                last_login: user.last_login,
                mfa_enabled: user.mfa_enabled,
            };
            
            (StatusCode::OK, Json(response))
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "user_not_found".to_string(),
                message: "User not found".to_string(),
            }),
        ),
    }
}

/// Delete user
pub async fn delete_user(
    State(state): State<UserState>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    let mut users = state.users.write().await;
    
    if let Some(pos) = users.iter().position(|u| u.id == user_id) {
        users.remove(pos);
        StatusCode::NO_CONTENT
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "user_not_found".to_string(),
                message: "User not found".to_string(),
            }),
        )
    }
}

/// Login
pub async fn login(
    State(state): State<UserState>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    let users = state.users.read().await;
    
    // Find user by username
    let user = match users.iter().find(|u| u.username == req.username) {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "invalid_credentials".to_string(),
                    message: "Invalid username or password".to_string(),
                }),
            );
        }
    };
    
    // Check if user is locked
    if let Some(locked_until) = user.locked_until {
        if locked_until > Utc::now() {
            return (
                StatusCode::LOCKED,
                Json(ErrorResponse {
                    error: "account_locked".to_string(),
                    message: "Account is temporarily locked".to_string(),
                }),
            );
        }
    }
    
    // Authenticate
    let credentials = Credentials {
        username: req.username.clone(),
        password: req.password.clone(),
    };
    
    match state.auth.authenticate(user, &credentials) {
        AuthResult::Success { user_id, session_id } => {
            // In production, you would create a real session here
            let token = format!("session_{}", session_id);
            let expires_at = Utc::now() + chrono::Duration::hours(24);
            
            let response = LoginResponse {
                user_id,
                session_id,
                token,
                expires_at,
            };
            
            (StatusCode::OK, Json(response))
        }
        AuthResult::Failure { reason } => {
            let status = match reason {
                AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
                AuthError::UserLocked => StatusCode::LOCKED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            
            (
                status,
                Json(ErrorResponse {
                    error: "authentication_failed".to_string(),
                    message: reason.to_string(),
                }),
            )
        }
    }
}

/// Create user router
pub fn create_user_router() -> Router {
    let state = UserState::new();
    
    Router::new()
        .route("/users", axum::routing::post(create_user))
        .route("/users", axum::routing::get(list_users))
        .route("/users/:id", axum::routing::get(get_user))
        .route("/users/:id", axum::routing::put(update_user))
        .route("/users/:id", axum::routing::delete(delete_user))
        .route("/login", axum::routing::post(login))
        .with_state(state)
}