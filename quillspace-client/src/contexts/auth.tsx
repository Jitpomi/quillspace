import { createContextId } from '@builder.io/qwik';

export interface AuthUser {
  id: string;
  email: string;
  first_name: string;
  last_name: string;
  role: string;
  tenant_id: string;
  [key: string]: any;
}

export interface AuthTenant {
  id: string;
  slug: string;
  name: string;
  [key: string]: any;
}

export interface AuthContext {
  user: AuthUser | null;
  tenant: AuthTenant | null;
}

export const AuthContextId = createContextId<AuthContext>('auth-context');
