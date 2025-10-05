/**
 * Environment Configuration for QuillSpace Frontend
 */

// API Configuration
export const API_CONFIG = {
  BASE_URL: 'http://localhost:3000/api/v1', // Default to local development
  TIMEOUT: 30000, // 30 seconds
};

// App Configuration
export const APP_CONFIG = {
  NAME: 'QuillSpace',
  VERSION: '1.0.0',
  DESCRIPTION: 'Multi-tenant Content Management System',
};

// Feature Flags
export const FEATURES = {
  ANALYTICS_ENABLED: true,
  REAL_TIME_UPDATES: true,
  DARK_MODE: false,
};

// Development helpers
export const isDevelopment = true; // Default to development
export const isProduction = false;
