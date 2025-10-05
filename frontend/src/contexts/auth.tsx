/**
 * Authentication Context for QuillSpace
 * Manages user authentication state and JWT tokens
 */

import { component$, createContextId, useContextProvider, useContext, useSignal, useVisibleTask$, Slot, $ } from '@builder.io/qwik';
import { api, setAuthToken, setTenantId, clearAuth, getAuthToken, getTenantId, type User, type Tenant } from '../services/api';

// Auth context interface
export interface AuthState {
  user: User | null;
  tenant: Tenant | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
}

export interface AuthActions {
  // eslint-disable-next-line no-unused-vars
  login(email: string, password: string): Promise<void>;
  // eslint-disable-next-line no-unused-vars
  register(userData: {
    email: string;
    password: string;
    name: string;
    tenant_name: string;
  }): Promise<void>;
  logout(): void;
  clearError(): void;
}

// Create context
export const AuthContext = createContextId<{
  state: AuthState;
  actions: AuthActions;
}>('auth-context');

// Auth provider component
export const AuthProvider = component$(() => {
  // State signals
  const user = useSignal<User | null>(null);
  const tenant = useSignal<Tenant | null>(null);
  const isAuthenticated = useSignal(false);
  const isLoading = useSignal(true);
  const error = useSignal<string | null>(null);

  // Auth actions
  const login = $(async (email: string, password: string) => {
    try {
      isLoading.value = true;
      error.value = null;

      const response = await api.login(email, password);
      
      // Set tokens
      setAuthToken(response.token);
      setTenantId(response.tenant.id);
      
      // Update state
      user.value = response.user;
      tenant.value = response.tenant;
      isAuthenticated.value = true;
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Login failed';
      isAuthenticated.value = false;
    } finally {
      isLoading.value = false;
    }
  });

  const register = $(async (userData: {
    email: string;
    password: string;
    name: string;
    tenant_name: string;
  }) => {
    try {
      isLoading.value = true;
      error.value = null;

      const response = await api.register(userData);
      
      // Set tokens
      setAuthToken(response.token);
      setTenantId(response.tenant.id);
      
      // Update state
      user.value = response.user;
      tenant.value = response.tenant;
      isAuthenticated.value = true;
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Registration failed';
      isAuthenticated.value = false;
    } finally {
      isLoading.value = false;
    }
  });

  const logout = $(() => {
    clearAuth();
    user.value = null;
    tenant.value = null;
    isAuthenticated.value = false;
    error.value = null;
  });

  const clearError = $(() => {
    error.value = null;
  });

  // Check for existing authentication on mount (browser-only)
  // eslint-disable-next-line qwik/no-use-visible-task
  useVisibleTask$(async () => {
    const token = getAuthToken();
    const tenantId = getTenantId();

    if (token && tenantId) {
      try {
        // Verify token is still valid
        const currentUser = await api.getCurrentUser();
        const currentTenant = await api.getCurrentTenant();
        
        user.value = currentUser;
        tenant.value = currentTenant;
        isAuthenticated.value = true;
      } catch (err) {
        // Token is invalid, clear auth
        clearAuth();
        isAuthenticated.value = false;
      }
    } else {
      isAuthenticated.value = false;
    }
    
    isLoading.value = false;
  });

  // Provide context
  useContextProvider(AuthContext, {
    state: {
      user: user.value,
      tenant: tenant.value,
      isAuthenticated: isAuthenticated.value,
      isLoading: isLoading.value,
      error: error.value,
    },
    actions: {
      login,
      register,
      logout,
      clearError,
    },
  });

  return <Slot />;
});

// Hook to use auth context
export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within AuthProvider');
  }
  return context;
};
