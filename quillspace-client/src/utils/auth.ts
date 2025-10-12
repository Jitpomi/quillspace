import type { Cookie } from '@builder.io/qwik-city';
import type { JwtClaims } from '~/api/schema';

/**
 * Authentication utilities for QuillSpace
 */

/**
 * Check if user is authenticated by verifying the presence of auth token in cookies
 */
export function isAuthenticated(cookie: Cookie): boolean {
  const authToken = cookie.get('auth_token');
  return !!authToken?.value;
}

/**
 * Get the authentication token from cookies
 */
export function getAuthToken(cookie: Cookie): string | null {
  const authToken = cookie.get('auth_token');
  return authToken?.value || null;
}

/**
 * Parse JWT claims from token (basic parsing without verification)
 * Note: This is for client-side convenience only. Server-side verification should be done on the backend.
 */
export function parseJwtClaims(token: string): JwtClaims | null {
  try {
    const parts = token.split('.');
    if (parts.length !== 3) return null;
    
    const payload = parts[1];
    const decoded = atob(payload.replace(/-/g, '+').replace(/_/g, '/'));
    return JSON.parse(decoded) as JwtClaims;
  } catch {
    return null;
  }
}

/**
 * Get user info from auth token
 */
export function getUserFromToken(cookie: Cookie): JwtClaims | null {
  const token = getAuthToken(cookie);
  if (!token) return null;
  
  return parseJwtClaims(token);
}

/**
 * Get user info from user_info cookie
 */
export function getUserInfo(cookie: Cookie): any | null {
  const userInfo = cookie.get('user_info');
  if (!userInfo?.value) return null;
  
  try {
    return JSON.parse(userInfo.value);
  } catch {
    return null;
  }
}

/**
 * Get tenant info from tenant_info cookie
 */
export function getTenantInfo(cookie: Cookie): any | null {
  const tenantInfo = cookie.get('tenant_info');
  if (!tenantInfo?.value) return null;
  
  try {
    return JSON.parse(tenantInfo.value);
  } catch {
    return null;
  }
}

/**
 * Get the appropriate redirect path for authenticated users
 */
export function getAuthenticatedUserPath(cookie: Cookie): string {
  const userInfo = getUserInfo(cookie);
  const tenantInfo = getTenantInfo(cookie);
  
  if (userInfo && tenantInfo) {
    return `/tenants/${tenantInfo.slug}/users/${userInfo.first_name}`;
  }
  
  // Fallback to home page if user/tenant info is not available
  return '/';
}

/**
 * Clear authentication token
 */
export function clearAuthToken(cookie: Cookie): void {
  cookie.delete('auth_token', { path: '/' });
}

/**
 * Complete logout - clear all auth-related cookies and redirect
 */
export function logout(cookie: Cookie): void {
  // Clear all authentication-related cookies
  cookie.delete('auth_token', { path: '/' });
  cookie.delete('user_info', { path: '/' });
  cookie.delete('tenant_info', { path: '/' });
}

/**
 * Set authentication token (for testing purposes)
 */
export function setAuthToken(cookie: Cookie, token: string): void {
  cookie.set('auth_token', token, { 
    path: '/',
    httpOnly: true,
    secure: true,
    sameSite: 'strict',
    maxAge: 60 * 60 * 24 * 7 // 7 days
  });
}
