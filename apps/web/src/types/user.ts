export enum Role {
  Admin = 'admin',
  Analyst = 'analyst',
  Viewer = 'viewer',
}

export interface User {
  id: string;
  username: string;
  email: string;
  role: Role;
  created_at: string;
  updated_at: string;
  last_login: string | null;
  failed_login_attempts: number;
  locked_until: string | null;
  mfa_enabled: boolean;
}