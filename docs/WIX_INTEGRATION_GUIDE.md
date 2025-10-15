# QuillSpace Wix Integration - Complete Guide

## 🎯 Overview

This integration allows users to **edit their existing Wix websites directly from QuillSpace** using the Puck visual editor. Users can connect their Wix sites, edit content through QuillSpace's interface, and publish changes back to Wix.

## 🏗️ Architecture

### **Complete Integration Flow:**
```
Wix Website ↔ Wix API ↔ QuillSpace Backend ↔ Puck Editor ↔ User
```

1. **Connect**: User provides Wix API credentials
2. **Sync**: QuillSpace fetches Wix site data via API
3. **Edit**: Wix content converted to Puck format for visual editing
4. **Save**: Puck data converted back to Wix format and saved via API
5. **Publish**: Changes published to live Wix website

## 🔧 Implementation Components

### **1. Backend Services**

#### **WixApiClient** (`wix_api.rs`)
- **Purpose**: Direct integration with Wix APIs
- **Features**:
  - Site management (get sites, site details)
  - Page content (get/update pages and components)
  - Publishing (publish site changes)
  - Authentication testing

#### **WixIntegrationService** (`wix_api.rs`)
- **Purpose**: Convert between Wix and Puck formats
- **Features**:
  - Wix → Puck conversion for editing
  - Puck → Wix conversion for saving
  - Component mapping (Text, Image, Button, etc.)

#### **ConnectedWebsitesService** (`connected_websites.rs`)
- **Purpose**: Manage connected websites and editing workflow
- **Key Methods**:
  - `load_wix_page_for_editing()` - Load Wix page as Puck data
  - `save_wix_page_from_quillspace()` - Save Puck edits to Wix
  - `publish_wix_site()` - Publish changes to live site
  - `test_wix_connection()` - Validate API credentials

### **2. API Endpoints**

```rust
// Website management
GET    /api/connected-websites                    // List connected sites
POST   /api/connected-websites                   // Connect via API
POST   /api/connected-websites/add-existing      // Add existing site

// Wix editing endpoints
GET    /api/connected-websites/wix/:site_id/pages/:page_id/edit   // Load for editing
PUT    /api/connected-websites/wix/:site_id/pages/:page_id/save   // Save changes
POST   /api/connected-websites/wix/:site_id/publish               // Publish site
POST   /api/connected-websites/wix/test-connection                // Test API
```

### **3. Database Schema**

```sql
-- Store encrypted Wix API credentials
website_builder_credentials (
  id UUID,
  user_id UUID,
  builder_type 'wix',
  encrypted_credentials TEXT, -- JSON: {"api_key": "...", "account_id": "..."}
  is_active BOOLEAN
)

-- Track connected Wix websites
connected_websites (
  id UUID,
  user_id UUID,
  builder_type 'wix',
  external_site_id TEXT,      -- Wix site ID
  name TEXT,                  -- Site display name
  url TEXT,                   -- Live site URL
  status 'active',
  metadata JSONB              -- Wix-specific data
)
```

### **4. Frontend Components**

#### **ConnectedWebsites Component**
- Shows list of connected websites
- **Edit button** for Wix sites → opens QuillSpace editor
- Add existing website modal

#### **Wix Editor Page** (`/editor/wix/[website_id]`)
- Full-screen Puck editor interface
- Save draft / Publish buttons
- Live preview link
- Converts Wix content ↔ Puck format

## 🔑 Wix API Requirements

### **Required Credentials:**
- **API Key**: Wix personal access token
- **Account ID**: Wix account identifier
- **Site ID**: Specific Wix site identifier

### **Required Wix APIs:**
- **Site List API**: Get user's sites
- **Pages API**: Get/update page content
- **Editor API**: Get/update page components
- **Site Actions API**: Publish changes

### **Permissions Needed:**
- Read site information
- Read/write page content
- Read/write page components
- Publish site changes

## 📋 Setup Instructions

### **1. Get Wix API Credentials**

#### **For Yasin's Existing Site:**
1. **Login to Wix** with the account that owns the site
2. **Go to Wix Developers** (developers.wix.com)
3. **Create API Key** with required permissions
4. **Get Account ID** from account settings
5. **Get Site ID** from the dashboard URL:
   ```
   https://manage.wix.com/dashboard/1e4a0091-f1d5-4a4c-a66e-4d09a75b4e9/...
                                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                                   This is the Site ID
   ```

### **2. Connect Website to QuillSpace**

#### **Option A: Manual Connection (Recommended)**
```bash
# Run the database script with Yasin's details
psql $DATABASE_URL -f scripts/add-yasin-website.sql
```

#### **Option B: API Connection**
```bash
curl -X POST /api/connected-websites \
  -H "Content-Type: application/json" \
  -d '{
    "builder_type": "wix",
    "credentials": {
      "api_key": "your_wix_api_key",
      "account_id": "your_wix_account_id"
    }
  }'
```

#### **Option C: Frontend Modal**
1. User goes to Websites page
2. Clicks "Add Existing Website"
3. Selects "Wix" and enters credentials
4. System automatically syncs their sites

### **3. Test the Integration**

1. **Connect Yasin's website** using one of the methods above
2. **Login as Yasin** and go to Websites page
3. **Verify website appears** in Connected Websites section
4. **Click Edit button** → should open QuillSpace editor
5. **Make test changes** and save
6. **Publish changes** to live Wix site

## 🎨 User Experience

### **Connected Websites View:**
```
┌─────────────────────────────────────────────┐
│ 1 website connected    [Add Existing Website] │
│                                             │
│ ┌─────────────────────────────────────────┐ │
│ │ Yasin Kakande                    ● Active│ │
│ │ Builder: Wix                            │ │
│ │ Domain: yasinkakande.wixsite.com        │ │
│ │ Visit Website ↗                         │ │
│ │                [✏️ Edit] [🔄] [⚙️] [🗑️] │ │
│ └─────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

### **Wix Editor Interface:**
```
┌─────────────────────────────────────────────┐
│ ← Yasin Kakande • Editing via QuillSpace    │
│                    [Save Draft] [Publish]   │
├─────────────────────────────────────────────┤
│                                             │
│         🎨 Puck Visual Editor               │
│                                             │
│    ┌─────────────────────────────────┐     │
│    │  Drag & Drop Components         │     │
│    │  • Text blocks                  │     │
│    │  • Images                       │     │
│    │  • Buttons                      │     │
│    │  • Containers                   │     │
│    └─────────────────────────────────┘     │
│                                             │
└─────────────────────────────────────────────┘
```

## 🔄 Data Flow

### **Loading for Edit:**
1. User clicks "Edit" on Wix website
2. QuillSpace loads Wix page via API
3. Wix components converted to Puck format
4. Puck editor displays editable content

### **Saving Changes:**
1. User makes changes in Puck editor
2. Puck data converted back to Wix format
3. Changes saved to Wix via API
4. User can preview or publish

### **Publishing:**
1. User clicks "Publish"
2. QuillSpace calls Wix publish API
3. Changes go live on Wix website
4. Success confirmation shown

## 🛡️ Security & Permissions

### **Credential Security:**
- API keys encrypted in database
- Credentials never exposed to frontend
- Secure API communication only

### **User Permissions:**
- Users can only edit their own connected sites
- Row-level security enforces isolation
- Admin override capabilities available

### **API Rate Limits:**
- Respect Wix API rate limits
- Implement retry logic for failed requests
- Cache frequently accessed data

## 🚀 Benefits

### **For Users:**
- **Unified Interface**: Edit all websites from QuillSpace
- **Visual Editing**: Puck editor is more intuitive than Wix editor
- **No Platform Lock-in**: Keep existing Wix site while using QuillSpace tools
- **Enhanced Features**: Access QuillSpace's advanced features

### **For QuillSpace:**
- **Competitive Advantage**: Unique capability no other platform offers
- **User Retention**: Users don't need to migrate away from Wix
- **Market Expansion**: Attract existing Wix users
- **Revenue Opportunity**: Premium feature for advanced editing

## 🔧 Technical Considerations

### **Component Mapping:**
- **Wix Text** → **Puck Text** component
- **Wix Image** → **Puck Image** component  
- **Wix Button** → **Puck Button** component
- **Wix Container** → **Puck Container** component
- **Custom Components** → **Puck Custom** component

### **Limitations:**
- Some Wix-specific features may not be editable
- Complex Wix apps might not convert perfectly
- Real-time collaboration limited by Wix API
- Publishing requires Wix API call (not instant)

### **Future Enhancements:**
- **Real-time Preview**: Live preview of changes
- **Advanced Components**: Support for more Wix component types
- **Bulk Operations**: Edit multiple pages simultaneously
- **Template Sync**: Sync Wix templates to QuillSpace

This integration transforms QuillSpace from just a website builder into a **universal website editor** that works with existing platforms! 🌟
