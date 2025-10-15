# Connecting Existing Websites to QuillSpace

This guide explains how to connect existing websites (like Yasin's Wix site) to QuillSpace so they appear in the user's "Connected Websites" section.

## 🎯 Overview

QuillSpace can connect to existing websites from various platforms:
- **Wix** - Via site ID and URL
- **WordPress** - Via REST API credentials  
- **Squarespace** - Via API key
- **Manual Connection** - For any website

## 🔧 Implementation Steps

### 1. Database Setup

Run the migration to create the connected websites tables:

```bash
# Run the migration
psql $DATABASE_URL -f quillspace-core/migrations/009_connected_websites.sql
```

### 2. Backend Services

The following services handle connected websites:

- **`ConnectedWebsitesService`** - Core business logic
- **`connected_websites.rs` routes** - API endpoints
- **Database tables** - Storage with RLS security

### 3. Frontend Components

- **`ConnectedWebsites`** - Displays connected websites
- **`AddExistingWebsiteModal`** - Modal for adding existing sites
- **API integration** - Loads websites from backend

## 📋 Adding Yasin's Wix Website

### Method 1: Database Script (Recommended for existing sites)

1. **Update the user email** in `scripts/add-yasin-website.sql`:
   ```sql
   WHERE email = 'yasin@blackwritersink.com' -- Update this email
   ```

2. **Run the script**:
   ```bash
   psql $DATABASE_URL -f scripts/add-yasin-website.sql
   ```

3. **Verify the connection**:
   ```sql
   SELECT name, url, status FROM connected_websites 
   WHERE builder_type = 'wix';
   ```

### Method 2: Frontend Modal (For users to add their own)

1. **User clicks "Add Existing Website"** in the Connected Websites section
2. **Fills out the form**:
   - Builder Type: Wix
   - Website Name: Yasin Kakande
   - Website URL: https://yasinkakande.wixsite.com/yasin-kakande
   - Wix Site ID: 1e4a0091-f1d5-4a4c-a66e-4d09a75b4e9
3. **Submits the form** - Website appears immediately

### Method 3: API Call (For programmatic addition)

```bash
curl -X POST /api/connected-websites/add-existing \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $USER_TOKEN" \
  -d '{
    "builder_type": "wix",
    "name": "Yasin Kakande",
    "url": "https://yasinkakande.wixsite.com/yasin-kakande",
    "domain": "yasinkakande.wixsite.com",
    "external_site_id": "1e4a0091-f1d5-4a4c-a66e-4d09a75b4e9"
  }'
```

## 🔍 Finding Wix Site Information

### From Wix Dashboard URL:
```
https://manage.wix.com/dashboard/1e4a0091-f1d5-4a4c-a66e-4d09a75b4e9/website-channel/referrals-info=sidebar
                                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                                 This is the Wix Site ID
```

### From Published Website URL:
```
https://yasinkakande.wixsite.com/yasin-kakande
       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
       Domain: yasinkakande.wixsite.com
```

## 📊 Database Schema

### Connected Websites Table:
```sql
connected_websites (
  id UUID PRIMARY KEY,
  tenant_id UUID,
  user_id UUID,
  builder_type builder_type, -- 'wix', 'wordpress', etc.
  external_site_id TEXT,     -- Site ID from external platform
  name TEXT,                 -- Display name
  url TEXT,                  -- Public website URL
  domain TEXT,               -- Domain name
  status connection_status,   -- 'active', 'inactive', etc.
  metadata JSONB             -- Platform-specific data
)
```

### Website Builder Credentials:
```sql
website_builder_credentials (
  id UUID PRIMARY KEY,
  user_id UUID,
  builder_type builder_type,
  encrypted_credentials TEXT, -- Encrypted API keys/passwords
  is_active BOOLEAN
)
```

## 🔐 Security Features

- **Row-Level Security (RLS)** - Users can only see their own websites
- **Tenant Isolation** - Multi-tenant security at database level
- **Encrypted Credentials** - API keys and passwords are encrypted
- **Manual Connections** - No credentials required for display-only connections

## 🎨 User Experience

### Empty State:
```
┌─────────────────────────────────────────────┐
│              🌐                             │
│        No websites connected yet            │
│                                             │
│  Select a website builder above to connect  │
│  your first website, or add an existing one │
│                                             │
│        [Add Existing Website]               │
└─────────────────────────────────────────────┘
```

### Connected State:
```
┌─────────────────────────────────────────────┐
│ 1 website connected    [Add Existing Website] │
│                                             │
│ ┌─────────────────────────────────────────┐ │
│ │ Yasin Kakande                    ● Active│ │
│ │ Builder: Wix                            │ │
│ │ Domain: yasinkakande.wixsite.com        │ │
│ │ Visit Website ↗                         │ │
│ │                    [✏️] [🔄] [⚙️] [🗑️] │ │
│ └─────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

## 🚀 API Endpoints

- `GET /api/connected-websites` - List user's connected websites
- `POST /api/connected-websites` - Connect via API credentials
- `POST /api/connected-websites/add-existing` - Add existing website manually
- `PUT /api/connected-websites/:id/refresh` - Refresh website data
- `DELETE /api/connected-websites/:id` - Disconnect website

## 🧪 Testing

### Test the Connection:
1. **Run the database migration**
2. **Add Yasin's website** using one of the methods above
3. **Login as Yasin** and navigate to the websites page
4. **Verify the website appears** in the Connected Websites section
5. **Test the "Visit Website" link** opens the Wix site

### Expected Result:
Yasin should see his Wix website listed under "Your Connected Websites" with:
- ✅ Website name: "Yasin Kakande"
- ✅ Status: Active (green dot)
- ✅ Builder: Wix
- ✅ Domain: yasinkakande.wixsite.com
- ✅ Working "Visit Website" link

This creates a seamless experience where users can see all their websites (QuillSpace-built and external) in one place! 🌟
