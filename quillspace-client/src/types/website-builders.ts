export interface WebsiteBuilder {
  id: string;
  name: string;
  description: string;
  icon: string;
  color: string;
  isConnected: boolean;
  apiEndpoint?: string;
  authType: 'api_key' | 'oauth' | 'username_password' | 'native';
  requiredFields: string[];
}

export interface WebsiteBuilderCredentials {
  builderId: string;
  credentials: Record<string, string>;
  isActive: boolean;
  lastSync?: Date;
  createdAt: Date;
}

export interface ConnectedWebsite {
  id: string;
  builderId: string;
  builderName: string;
  name: string;
  url?: string;
  domain?: string;
  status: 'active' | 'inactive' | 'syncing' | 'error';
  lastSync?: Date;
  createdAt: Date;
  updatedAt: Date;
  metadata?: Record<string, any>;
}

export interface WebsiteBuilderConfig {
  jflux: {
    authType: 'native';
    requiredFields: [];
    description: 'Native QuillSpace website builder';
  };
  wix: {
    authType: 'api_key';
    requiredFields: ['apiKey', 'siteId'];
    description: 'Connect via Wix API';
    apiEndpoint: 'https://www.wixapis.com';
  };
  wordpress: {
    authType: 'username_password';
    requiredFields: ['username', 'password', 'siteUrl'];
    description: 'Connect via WordPress REST API';
  };
  squarespace: {
    authType: 'api_key';
    requiredFields: ['apiKey'];
    description: 'Connect via Squarespace API';
    apiEndpoint: 'https://api.squarespace.com';
  };
}

export type BuilderType = keyof WebsiteBuilderConfig;

export interface WebsiteBuilderService {
  testConnection(credentials: Record<string, string>): Promise<boolean>;
  syncWebsites(credentials: Record<string, string>): Promise<ConnectedWebsite[]>;
  publishContent(websiteId: string, content: any): Promise<boolean>;
  getWebsiteInfo(websiteId: string): Promise<ConnectedWebsite>;
}
