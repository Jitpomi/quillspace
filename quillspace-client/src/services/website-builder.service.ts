import type { 
  WebsiteBuilder, 
  WebsiteBuilderCredentials, 
  ConnectedWebsite, 
  BuilderType 
} from '~/types/website-builders';

export class WebsiteBuilderService {
  private static instance: WebsiteBuilderService;
  private baseUrl = '/api/website-builders';

  static getInstance(): WebsiteBuilderService {
    if (!WebsiteBuilderService.instance) {
      WebsiteBuilderService.instance = new WebsiteBuilderService();
    }
    return WebsiteBuilderService.instance;
  }

  /**
   * Get all available website builders
   */
  async getAvailableBuilders(): Promise<WebsiteBuilder[]> {
    const response = await fetch(`${this.baseUrl}/available`);
    if (!response.ok) {
      throw new Error('Failed to fetch available builders');
    }
    return response.json();
  }

  /**
   * Test connection to a website builder
   */
  async testConnection(builderId: BuilderType, credentials: Record<string, string>): Promise<boolean> {
    const response = await fetch(`${this.baseUrl}/${builderId}/test`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ credentials }),
    });

    if (!response.ok) {
      throw new Error('Failed to test connection');
    }

    const result = await response.json();
    return result.success;
  }

  /**
   * Connect a website builder
   */
  async connectBuilder(builderId: BuilderType, credentials: Record<string, string>): Promise<WebsiteBuilderCredentials> {
    const response = await fetch(`${this.baseUrl}/${builderId}/connect`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ credentials }),
    });

    if (!response.ok) {
      throw new Error('Failed to connect builder');
    }

    return response.json();
  }

  /**
   * Disconnect a website builder
   */
  async disconnectBuilder(builderId: BuilderType): Promise<void> {
    const response = await fetch(`${this.baseUrl}/${builderId}/disconnect`, {
      method: 'DELETE',
    });

    if (!response.ok) {
      throw new Error('Failed to disconnect builder');
    }
  }

  /**
   * Get connected websites
   */
  async getConnectedWebsites(): Promise<ConnectedWebsite[]> {
    const response = await fetch(`${this.baseUrl}/websites`);
    if (!response.ok) {
      throw new Error('Failed to fetch connected websites');
    }
    return response.json();
  }

  /**
   * Sync websites from a specific builder
   */
  async syncWebsites(builderId: BuilderType): Promise<ConnectedWebsite[]> {
    const response = await fetch(`${this.baseUrl}/${builderId}/sync`, {
      method: 'POST',
    });

    if (!response.ok) {
      throw new Error('Failed to sync websites');
    }

    return response.json();
  }

  /**
   * Publish content to a website
   */
  async publishContent(websiteId: string, content: any): Promise<boolean> {
    const response = await fetch(`${this.baseUrl}/websites/${websiteId}/publish`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ content }),
    });

    if (!response.ok) {
      throw new Error('Failed to publish content');
    }

    const result = await response.json();
    return result.success;
  }

  /**
   * Update website settings
   */
  async updateWebsite(websiteId: string, updates: Partial<ConnectedWebsite>): Promise<ConnectedWebsite> {
    const response = await fetch(`${this.baseUrl}/websites/${websiteId}`, {
      method: 'PATCH',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(updates),
    });

    if (!response.ok) {
      throw new Error('Failed to update website');
    }

    return response.json();
  }

  /**
   * Delete a connected website
   */
  async deleteWebsite(websiteId: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/websites/${websiteId}`, {
      method: 'DELETE',
    });

    if (!response.ok) {
      throw new Error('Failed to delete website');
    }
  }

  /**
   * Get website builder specific configuration
   */
  getBuilderConfig(builderId: BuilderType) {
    const configs = {
      jflux: {
        authType: 'native' as const,
        requiredFields: [],
        description: 'Native QuillSpace website builder',
        icon: '🚀',
        color: 'from-blue-500 to-purple-600'
      },
      wix: {
        authType: 'api_key' as const,
        requiredFields: ['apiKey', 'siteId'],
        description: 'Connect via Wix API',
        icon: '🎨',
        color: 'from-orange-500 to-red-500',
        apiEndpoint: 'https://www.wixapis.com'
      },
      wordpress: {
        authType: 'username_password' as const,
        requiredFields: ['username', 'password', 'siteUrl'],
        description: 'Connect via WordPress REST API',
        icon: '📝',
        color: 'from-blue-600 to-blue-800'
      },
      squarespace: {
        authType: 'api_key' as const,
        requiredFields: ['apiKey'],
        description: 'Connect via Squarespace API',
        icon: '⬛',
        color: 'from-gray-700 to-black',
        apiEndpoint: 'https://api.squarespace.com'
      }
    };

    return configs[builderId];
  }
}
