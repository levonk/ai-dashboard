use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use analytics_rs::auth::{SessionManager, SessionStore, Permission, UserId};
use std::sync::Arc;

/// Authentication state
#[derive(Clone)]
pub struct AuthState {
    session_manager: Arc<SessionManager>,
}

impl AuthState {
    pub fn new() -> Self {
        let _session_store = SessionStore::new();
        let session_manager = Arc::new(SessionManager::new());
        
        Self {
            session_manager,
        }
    }
}

impl Default for AuthState {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract user ID from request
#[derive(Clone)]
pub struct AuthenticatedUser {
    pub user_id: UserId,
    pub session_id: uuid::Uuid,
}

/// Authentication middleware
pub async fn auth_middleware(
    State(state): State<AuthState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Parse Bearer token
    let token = if auth_header.starts_with("Bearer ") {
        auth_header[7..].to_string()
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    // Validate session
    let session = state
        .session_manager
        .validate_session(&token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Add user info to request extensions
    request.extensions_mut().insert(AuthenticatedUser {
        user_id: session.user_id,
        session_id: session.id,
    });

    Ok(next.run(request).await)
}

/// Require specific permission middleware
pub fn require_permission(_permission: Permission) -> impl Fn(Request, Next) -> futures::future::BoxFuture<'static, Result<Response, StatusCode>> + Clone {
    move |request: Request, next: Next| {
        Box::pin(async move {
            // Get authenticated user from request extensions
            let _auth_user = request
                .extensions()
                .get::<AuthenticatedUser>()
                .ok_or(StatusCode::UNAUTHORIZED)?;

            // In production, you would check permissions here
            // For now, we'll just allow authenticated users
            Ok(next.run(request).await)
        })
    }
}

/// Require admin role middleware
pub fn require_admin() -> impl Fn(Request, Next) -> futures::future::BoxFuture<'static, Result<Response, StatusCode>> + Clone {
    move |request: Request, next: Next| {
        Box::pin(async move {
            // Get authenticated user from request extensions
            let _auth_user = request
                .extensions()
                .get::<AuthenticatedUser>()
                .ok_or(StatusCode::UNAUTHORIZED)?;

            // In production, you would check if user is admin here
            // For now, we'll just allow authenticated users
            Ok(next.run(request).await)
        })
    }
}

/// Optional authentication middleware (doesn't fail if no auth)
pub async fn optional_auth_middleware(
    State(state): State<AuthState>,
    mut request: Request,
    next: Next,
) -> Response {
    // Try to extract authorization header
    if let Some(auth_header) = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
    {
        // Parse Bearer token
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            // Try to validate session
            if let Ok(session) = state.session_manager.validate_session(token) {
                // Add user info to request extensions
                request.extensions_mut().insert(AuthenticatedUser {
                    user_id: session.user_id,
                    session_id: session.id,
                });
            }
        }
    }

    next.run(request).await
}

/// Extract authenticated user from request
pub fn extract_auth_user(request: &Request) -> Option<&AuthenticatedUser> {
    request.extensions().get::<AuthenticatedUser>()
}

/// Check if request is authenticated
pub fn is_authenticated(request: &Request) -> bool {
    extract_auth_user(request).is_some()
}