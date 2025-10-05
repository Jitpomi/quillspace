/**
 * API Service Layer for QuillSpace Backend Communication
 * Handles all HTTP requests to the Rust backend
 */

import { API_CONFIG } from '../config/env';

// API Configuration
const API_BASE_URL = API_CONFIG.BASE_URL;

// Types matching the Rust backend
export interface User {
  id: string;
  email: string;
  name: string;
  role: 'admin' | 'editor' | 'viewer';
  tenant_id: string;
  created_at: string;
  updated_at: string;
  is_active: boolean;
}

export interface Content {
  id: string;
  title: string;
  slug: string;
  content: string;
  status: 'draft' | 'published' | 'archived';
  author_id: string;
  tenant_id: string;
  created_at: string;
  updated_at: string;
  published_at?: string;
}

export interface Tenant {
  id: string;
  name: string;
  slug: string;
  settings: {
    branding: {
      primary_color: string;
      logo_url?: string;
      favicon_url?: string;
    };
    features: {
      analytics_enabled: boolean;
      comments_enabled: boolean;
      seo_enabled: boolean;
    };
    security: {
      two_factor_required: boolean;
      password_policy: 'basic' | 'strong' | 'enterprise';
    };
  };
  created_at: string;
  updated_at: string;
  is_active: boolean;
}

export interface AnalyticsEvent {
  event_type: string;
  event_data: Record<string, any>;
  session_id?: string;
  user_id?: string;
  tenant_id: string;
  timestamp: string;
}

export interface AnalyticsMetrics {
  total_events: number;
  unique_users: number;
  page_views: number;
  content_published: number;
  growth_rate: number;
}

// Authentication token management
let authToken: string | null = null;
let currentTenantId: string | null = null;

// Safe localStorage access (browser-only)
const isBrowser = typeof window !== 'undefined';

export const setAuthToken = (token: string) => {
  authToken = token;
  if (isBrowser) {
    localStorage.setItem('quillspace_token', token);
  }
};

export const setTenantId = (tenantId: string) => {
  currentTenantId = tenantId;
  if (isBrowser) {
    localStorage.setItem('quillspace_tenant_id', tenantId);
  }
};

export const getAuthToken = (): string | null => {
  if (authToken) return authToken;
  if (isBrowser) {
    return localStorage.getItem('quillspace_token');
  }
  return null;
};

export const getTenantId = (): string | null => {
  if (currentTenantId) return currentTenantId;
  if (isBrowser) {
    return localStorage.getItem('quillspace_tenant_id');
  }
  return null;
};

export const clearAuth = () => {
  authToken = null;
  currentTenantId = null;
  if (isBrowser) {
    localStorage.removeItem('quillspace_token');
    localStorage.removeItem('quillspace_tenant_id');
  }
};

// HTTP client with authentication
class ApiClient {
  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const token = getAuthToken();
    const tenantId = getTenantId();

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...(options.headers as Record<string, string>),
    };

    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    if (tenantId) {
      headers['X-Tenant-ID'] = tenantId;
    }

    const response = await fetch(`${API_BASE_URL}${endpoint}`, {
      ...options,
      headers,
    });

    if (!response.ok) {
      if (response.status === 401) {
        clearAuth();
        throw new Error('Authentication required');
      }
      throw new Error(`API Error: ${response.status} ${response.statusText}`);
    }

    const result = await response.json();
    
    // Handle wrapped API response format
    if (result.success && result.data !== undefined) {
      return result.data;
    }
    
    // Handle direct response format (fallback)
    return result;
  }

  // Authentication endpoints
  async login(email: string, password: string): Promise<{ token: string; user: User; tenant: Tenant }> {
    return this.request('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ email, password }),
    });
  }

  async register(userData: {
    email: string;
    password: string;
    name: string;
    tenant_name: string;
  }): Promise<{ token: string; user: User; tenant: Tenant }> {
    return this.request('/auth/register', {
      method: 'POST',
      body: JSON.stringify(userData),
    });
  }

  async getCurrentUser(): Promise<User> {
    return this.request('/auth/me');
  }

  // Content endpoints
  async getContent(): Promise<Content[]> {
    return this.request('/content');
  }

  async getContentById(id: string): Promise<Content> {
    return this.request(`/content/${id}`);
  }

  async createContent(contentData: {
    title: string;
    slug: string;
    content: string;
    status: 'draft' | 'published';
  }): Promise<Content> {
    return this.request('/content', {
      method: 'POST',
      body: JSON.stringify(contentData),
    });
  }

  async updateContent(id: string, contentData: Partial<Content>): Promise<Content> {
    return this.request(`/content/${id}`, {
      method: 'PUT',
      body: JSON.stringify(contentData),
    });
  }

  async deleteContent(id: string): Promise<void> {
    return this.request(`/content/${id}`, {
      method: 'DELETE',
    });
  }

  async publishContent(id: string): Promise<Content> {
    return this.request(`/content/${id}/publish`, {
      method: 'POST',
    });
  }

  // User management endpoints
  async getUsers(): Promise<User[]> {
    return this.request('/users');
  }

  async getUserById(id: string): Promise<User> {
    return this.request(`/users/${id}`);
  }

  async createUser(userData: {
    email: string;
    name: string;
    role: 'admin' | 'editor' | 'viewer';
  }): Promise<User> {
    return this.request('/users', {
      method: 'POST',
      body: JSON.stringify(userData),
    });
  }

  async updateUser(id: string, userData: Partial<User>): Promise<User> {
    return this.request(`/users/${id}`, {
      method: 'PUT',
      body: JSON.stringify(userData),
    });
  }

  async deleteUser(id: string): Promise<void> {
    return this.request(`/users/${id}`, {
      method: 'DELETE',
    });
  }

  // Analytics endpoints
  async getAnalyticsMetrics(): Promise<AnalyticsMetrics> {
    return this.request('/analytics/metrics');
  }

  async recordEvent(eventData: {
    event_type: string;
    event_data: Record<string, any>;
    session_id?: string;
  }): Promise<void> {
    return this.request('/analytics/events', {
      method: 'POST',
      body: JSON.stringify(eventData),
    });
  }

  async getTopContent(): Promise<Array<{ title: string; views: number; }>> {
    return this.request('/analytics/top-content');
  }

  async getRecentActivity(): Promise<Array<{ type: string; description: string; timestamp: string; }>> {
    return this.request('/analytics/recent-activity');
  }

  // Tenant endpoints
  async getCurrentTenant(): Promise<Tenant> {
    return this.request('/tenants/current');
  }

  async updateTenantSettings(settings: Tenant['settings']): Promise<Tenant> {
    return this.request('/tenants/current/settings', {
      method: 'PUT',
      body: JSON.stringify(settings),
    });
  }
}

// Export singleton instance
export const apiClient = new ApiClient();

// Convenience functions
export const api = {
  // Auth
  login: apiClient.login.bind(apiClient),
  register: apiClient.register.bind(apiClient),
  getCurrentUser: apiClient.getCurrentUser.bind(apiClient),

  // Content
  getContent: apiClient.getContent.bind(apiClient),
  getContentById: apiClient.getContentById.bind(apiClient),
  createContent: apiClient.createContent.bind(apiClient),
  updateContent: apiClient.updateContent.bind(apiClient),
  deleteContent: apiClient.deleteContent.bind(apiClient),
  publishContent: apiClient.publishContent.bind(apiClient),

  // Users
  getUsers: apiClient.getUsers.bind(apiClient),
  getUserById: apiClient.getUserById.bind(apiClient),
  createUser: apiClient.createUser.bind(apiClient),
  updateUser: apiClient.updateUser.bind(apiClient),
  deleteUser: apiClient.deleteUser.bind(apiClient),

  // Analytics
  getAnalyticsMetrics: apiClient.getAnalyticsMetrics.bind(apiClient),
  recordEvent: apiClient.recordEvent.bind(apiClient),
  getTopContent: apiClient.getTopContent.bind(apiClient),
  getRecentActivity: apiClient.getRecentActivity.bind(apiClient),

  // Tenants
  getCurrentTenant: apiClient.getCurrentTenant.bind(apiClient),
  updateTenantSettings: apiClient.updateTenantSettings.bind(apiClient),
};
