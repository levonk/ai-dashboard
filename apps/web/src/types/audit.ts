export enum AuditEventType {
  UserLogin = 'user_login',
  UserLogout = 'user_logout',
  LoginFailed = 'login_failed',
  PasswordChanged = 'password_changed',
  PasswordResetRequested = 'password_reset_requested',
  MfaEnabled = 'mfa_enabled',
  MfaDisabled = 'mfa_disabled',
  UserCreated = 'user_created',
  UserUpdated = 'user_updated',
  UserDeleted = 'user_deleted',
  RoleAssigned = 'role_assigned',
  RoleRevoked = 'role_revoked',
  DataRead = 'data_read',
  DataExported = 'data_exported',
  DataDeleted = 'data_deleted',
  ConfigRead = 'config_read',
  ConfigUpdated = 'config_updated',
  SystemStarted = 'system_started',
  SystemStopped = 'system_stopped',
  SystemError = 'system_error',
  ApiAccess = 'api_access',
  ApiError = 'api_error',
}

export enum AuditSeverity {
  Info = 'info',
  Warning = 'warning',
  Error = 'error',
  Critical = 'critical',
}

export interface AuditEvent {
  id: string;
  timestamp: string;
  event_type: AuditEventType;
  severity: AuditSeverity;
  user_id: string | null;
  session_id: string | null;
  ip_address: string | null;
  user_agent: string | null;
  resource: string;
  action: string;
  details: Record<string, unknown>;
  success: boolean;
  error_message: string | null;
}