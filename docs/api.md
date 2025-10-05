# QuillSpace API Documentation

## Overview

The QuillSpace API is a RESTful service built with Rust and Axum, designed for high-performance multi-tenant content management. All API endpoints are tenant-scoped and require authentication.

**Base URL**: `https://api.quillspace.com/v1`  
**Authentication**: Bearer JWT tokens  
**Content-Type**: `application/json`

## Authentication

### Login

Authenticate a user and receive JWT tokens.

```http
POST /auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "secure_password"
}
```

**Response**:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 3600,
  "user": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "email": "user@example.com",
    "role": "admin",
    "tenant_id": "123e4567-e89b-12d3-a456-426614174001"
  }
}
```

### Refresh Token

Refresh an expired access token.

```http
POST /auth/refresh
Content-Type: application/json

{
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

### Logout

Invalidate the current session.

```http
POST /auth/logout
Authorization: Bearer <access_token>
```

## Error Handling

All API errors follow a consistent format:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid input data",
    "details": {
      "field": "email",
      "reason": "Invalid email format"
    },
    "request_id": "req_123456789"
  }
}
```

### HTTP Status Codes

| Code | Description |
|------|-------------|
| `200` | Success |
| `201` | Created |
| `400` | Bad Request |
| `401` | Unauthorized |
| `403` | Forbidden |
| `404` | Not Found |
| `409` | Conflict |
| `422` | Unprocessable Entity |
| `429` | Too Many Requests |
| `500` | Internal Server Error |

### Error Codes

| Code | Description |
|------|-------------|
| `VALIDATION_ERROR` | Input validation failed |
| `AUTHENTICATION_ERROR` | Invalid credentials |
| `AUTHORIZATION_ERROR` | Insufficient permissions |
| `TENANT_NOT_FOUND` | Tenant does not exist |
| `RESOURCE_NOT_FOUND` | Requested resource not found |
| `RATE_LIMIT_EXCEEDED` | Too many requests |
| `INTERNAL_ERROR` | Server error |

## Pagination

List endpoints support cursor-based pagination:

```http
GET /api/users?limit=20&cursor=eyJpZCI6IjEyMyIsImNyZWF0ZWRfYXQiOiIyMDIzLTEyLTAxVDEwOjAwOjAwWiJ9
```

**Response**:
```json
{
  "data": [...],
  "pagination": {
    "limit": 20,
    "has_more": true,
    "next_cursor": "eyJpZCI6IjQ1NiIsImNyZWF0ZWRfYXQiOiIyMDIzLTEyLTAxVDEwOjMwOjAwWiJ9"
  }
}
```

## Tenants

### Get Current Tenant

```http
GET /api/tenant
Authorization: Bearer <access_token>
```

**Response**:
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174001",
  "name": "Acme Publishing",
  "domain": "acme.quillspace.com",
  "settings": {
    "branding": {
      "logo_url": "https://cdn.quillspace.com/logos/acme.png",
      "primary_color": "#007bff"
    },
    "features": {
      "analytics_enabled": true,
      "custom_domains": true
    }
  },
  "created_at": "2023-01-15T10:00:00Z",
  "updated_at": "2023-12-01T15:30:00Z"
}
```

### Update Tenant Settings

```http
PATCH /api/tenant
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "name": "Acme Publishing Co.",
  "settings": {
    "branding": {
      "primary_color": "#28a745"
    }
  }
}
```

## Users

### List Users

```http
GET /api/users?limit=50&cursor=<cursor>&role=admin
Authorization: Bearer <access_token>
```

**Query Parameters**:
- `limit` (optional): Number of users to return (1-100, default: 20)
- `cursor` (optional): Pagination cursor
- `role` (optional): Filter by user role
- `search` (optional): Search by email or name

**Response**:
```json
{
  "data": [
    {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "email": "admin@acme.com",
      "first_name": "John",
      "last_name": "Doe",
      "role": "admin",
      "active": true,
      "last_login": "2023-12-01T09:15:00Z",
      "created_at": "2023-01-15T10:00:00Z"
    }
  ],
  "pagination": {
    "limit": 50,
    "has_more": false,
    "next_cursor": null
  }
}
```

### Get User

```http
GET /api/users/{user_id}
Authorization: Bearer <access_token>
```

### Create User

```http
POST /api/users
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "email": "newuser@acme.com",
  "first_name": "Jane",
  "last_name": "Smith",
  "role": "editor",
  "password": "secure_password"
}
```

### Update User

```http
PATCH /api/users/{user_id}
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "first_name": "Jane",
  "last_name": "Doe",
  "role": "admin"
}
```

### Delete User

```http
DELETE /api/users/{user_id}
Authorization: Bearer <access_token>
```

## Content Management

### List Content

```http
GET /api/content?type=article&status=published&limit=20
Authorization: Bearer <access_token>
```

**Query Parameters**:
- `type` (optional): Content type (article, page, listing)
- `status` (optional): Content status (draft, published, archived)
- `author_id` (optional): Filter by author
- `category_id` (optional): Filter by category
- `search` (optional): Full-text search
- `sort` (optional): Sort field (created_at, updated_at, title)
- `order` (optional): Sort order (asc, desc)

**Response**:
```json
{
  "data": [
    {
      "id": "123e4567-e89b-12d3-a456-426614174002",
      "type": "article",
      "title": "Getting Started with QuillSpace",
      "slug": "getting-started-with-quillspace",
      "excerpt": "Learn how to set up your first publishing workflow...",
      "status": "published",
      "author": {
        "id": "123e4567-e89b-12d3-a456-426614174000",
        "name": "John Doe"
      },
      "category": {
        "id": "123e4567-e89b-12d3-a456-426614174003",
        "name": "Tutorials"
      },
      "published_at": "2023-12-01T10:00:00Z",
      "created_at": "2023-11-30T15:00:00Z",
      "updated_at": "2023-12-01T09:45:00Z"
    }
  ],
  "pagination": {
    "limit": 20,
    "has_more": true,
    "next_cursor": "eyJpZCI6IjQ1NiJ9"
  }
}
```

### Get Content

```http
GET /api/content/{content_id}
Authorization: Bearer <access_token>
```

**Response**:
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174002",
  "type": "article",
  "title": "Getting Started with QuillSpace",
  "slug": "getting-started-with-quillspace",
  "content": "# Getting Started\n\nWelcome to QuillSpace...",
  "excerpt": "Learn how to set up your first publishing workflow...",
  "status": "published",
  "author_id": "123e4567-e89b-12d3-a456-426614174000",
  "category_id": "123e4567-e89b-12d3-a456-426614174003",
  "tags": ["tutorial", "getting-started"],
  "metadata": {
    "seo_title": "Getting Started with QuillSpace - Complete Guide",
    "seo_description": "Step-by-step guide to setting up QuillSpace...",
    "featured_image": "https://cdn.quillspace.com/images/tutorial-hero.jpg"
  },
  "published_at": "2023-12-01T10:00:00Z",
  "created_at": "2023-11-30T15:00:00Z",
  "updated_at": "2023-12-01T09:45:00Z"
}
```

### Create Content

```http
POST /api/content
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "type": "article",
  "title": "New Article Title",
  "content": "# Article Content\n\nThis is the article content...",
  "excerpt": "Brief description of the article...",
  "category_id": "123e4567-e89b-12d3-a456-426614174003",
  "tags": ["news", "updates"],
  "status": "draft",
  "metadata": {
    "seo_title": "Custom SEO Title",
    "featured_image": "https://cdn.quillspace.com/images/new-article.jpg"
  }
}
```

### Update Content

```http
PATCH /api/content/{content_id}
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "title": "Updated Article Title",
  "status": "published",
  "published_at": "2023-12-01T12:00:00Z"
}
```

### Delete Content

```http
DELETE /api/content/{content_id}
Authorization: Bearer <access_token>
```

## Categories

### List Categories

```http
GET /api/categories
Authorization: Bearer <access_token>
```

**Response**:
```json
{
  "data": [
    {
      "id": "123e4567-e89b-12d3-a456-426614174003",
      "name": "Tutorials",
      "slug": "tutorials",
      "description": "Step-by-step guides and tutorials",
      "parent_id": null,
      "content_count": 15,
      "created_at": "2023-01-15T10:00:00Z"
    }
  ]
}
```

### Create Category

```http
POST /api/categories
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "name": "News",
  "description": "Latest news and updates",
  "parent_id": null
}
```

## Analytics

### Get Content Analytics

```http
GET /api/analytics/content/{content_id}?period=30d
Authorization: Bearer <access_token>
```

**Query Parameters**:
- `period`: Time period (1d, 7d, 30d, 90d, 1y)
- `granularity`: Data granularity (hour, day, week, month)

**Response**:
```json
{
  "content_id": "123e4567-e89b-12d3-a456-426614174002",
  "period": "30d",
  "metrics": {
    "views": 1250,
    "unique_visitors": 890,
    "avg_time_on_page": 180,
    "bounce_rate": 0.35
  },
  "timeline": [
    {
      "date": "2023-12-01",
      "views": 45,
      "unique_visitors": 32
    }
  ]
}
```

### Get Dashboard Analytics

```http
GET /api/analytics/dashboard?period=7d
Authorization: Bearer <access_token>
```

**Response**:
```json
{
  "period": "7d",
  "summary": {
    "total_views": 5420,
    "unique_visitors": 3210,
    "published_content": 12,
    "active_users": 8
  },
  "top_content": [
    {
      "id": "123e4567-e89b-12d3-a456-426614174002",
      "title": "Getting Started with QuillSpace",
      "views": 450,
      "unique_visitors": 320
    }
  ],
  "traffic_sources": [
    {
      "source": "organic",
      "visitors": 1500,
      "percentage": 46.7
    },
    {
      "source": "direct",
      "visitors": 980,
      "percentage": 30.5
    }
  ]
}
```

## File Management

### Upload File

```http
POST /api/files/upload
Authorization: Bearer <access_token>
Content-Type: multipart/form-data

file: <binary_data>
folder: "images"
```

**Response**:
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174004",
  "filename": "hero-image.jpg",
  "original_name": "my-hero-image.jpg",
  "mime_type": "image/jpeg",
  "size": 245760,
  "url": "https://cdn.quillspace.com/files/123e4567-e89b-12d3-a456-426614174004/hero-image.jpg",
  "folder": "images",
  "created_at": "2023-12-01T10:30:00Z"
}
```

### List Files

```http
GET /api/files?folder=images&type=image&limit=20
Authorization: Bearer <access_token>
```

### Delete File

```http
DELETE /api/files/{file_id}
Authorization: Bearer <access_token>
```

## Webhooks

### List Webhooks

```http
GET /api/webhooks
Authorization: Bearer <access_token>
```

### Create Webhook

```http
POST /api/webhooks
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "url": "https://your-app.com/webhooks/quillspace",
  "events": ["content.published", "user.created"],
  "secret": "your_webhook_secret"
}
```

### Webhook Events

| Event | Description |
|-------|-------------|
| `content.created` | New content created |
| `content.updated` | Content updated |
| `content.published` | Content published |
| `content.deleted` | Content deleted |
| `user.created` | New user created |
| `user.updated` | User updated |
| `user.deleted` | User deleted |

**Webhook Payload Example**:
```json
{
  "event": "content.published",
  "timestamp": "2023-12-01T10:00:00Z",
  "tenant_id": "123e4567-e89b-12d3-a456-426614174001",
  "data": {
    "content": {
      "id": "123e4567-e89b-12d3-a456-426614174002",
      "title": "Getting Started with QuillSpace",
      "status": "published",
      "published_at": "2023-12-01T10:00:00Z"
    }
  }
}
```

## Rate Limiting

API requests are rate-limited per tenant:

- **Standard Plan**: 1,000 requests per hour
- **Pro Plan**: 10,000 requests per hour  
- **Enterprise Plan**: 100,000 requests per hour

Rate limit headers are included in all responses:

```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1701432000
```

## SDKs and Libraries

### Official SDKs

- **JavaScript/TypeScript**: `@quillspace/sdk-js`
- **Python**: `quillspace-python`
- **PHP**: `quillspace/php-sdk`
- **Go**: `github.com/quillspace/go-sdk`

### JavaScript SDK Example

```javascript
import { QuillSpaceClient } from '@quillspace/sdk-js';

const client = new QuillSpaceClient({
  apiKey: 'your_api_key',
  baseUrl: 'https://api.quillspace.com/v1'
});

// List content
const content = await client.content.list({
  type: 'article',
  status: 'published',
  limit: 10
});

// Create content
const newArticle = await client.content.create({
  type: 'article',
  title: 'My New Article',
  content: '# Hello World\n\nThis is my first article.',
  status: 'draft'
});
```

## OpenAPI Specification

The complete OpenAPI 3.0 specification is available at:
- **JSON**: `https://api.quillspace.com/v1/openapi.json`
- **YAML**: `https://api.quillspace.com/v1/openapi.yaml`
- **Interactive Docs**: `https://api.quillspace.com/docs`

## Support

- **Documentation**: https://docs.quillspace.com
- **API Status**: https://status.quillspace.com
- **Support Email**: api-support@quillspace.com
- **Community Forum**: https://community.quillspace.com

For technical support, please include:
- Request ID from error responses
- Complete request/response details
- Tenant ID and user ID (if applicable)
- Timestamp of the issue
