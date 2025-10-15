# QuillSpace Wix Integration Setup

## 🎯 Overview

This setup allows QuillSpace to automatically show client websites that were built by the QuillSpace team using your Wix account. When clients like Yasin log in, they'll see their websites in the "Connected Websites" section and can edit them directly through QuillSpace.

## 🔧 Setup Steps

### 1. Get Your Wix API Credentials

1. **Login to Wix** with your QuillSpace account
2. **Go to Wix Developers**: https://developers.wix.com/
3. **Create a new app** or use existing one
4. **Get your credentials**:
   - **API Key**: Your personal access token
   - **Account ID**: Your Wix account identifier

### 2. Configure Environment Variables

Add these to your `.env` file:

```bash
# QuillSpace Wix Account (for client websites)
QUILLSPACE_WIX_API_KEY=your_actual_wix_api_key
QUILLSPACE_WIX_ACCOUNT_ID=your_actual_wix_account_id
```

### 3. Run Database Migration

```bash
# Create the connected websites tables
psql $DATABASE_URL -f quillspace-core/migrations/009_connected_websites.sql
```

### 4. Test the Integration

1. **Start QuillSpace**: `cargo run` or your usual dev command
2. **Login as Yasin** (or any client with a website)
3. **Go to Websites page**: Should show loading state
4. **Check Connected Websites**: Should show Yasin's website from Wix

## 🔍 How It Works

### **Automatic Website Detection:**

1. **User visits Websites page**
2. **QuillSpace calls Wix API** using your credentials
3. **Gets all sites** from your Wix account
4. **Matches sites to users** by name (e.g., "Yasin Kakande" site → Yasin user)
5. **Shows matched websites** in Connected Websites section

### **Matching Logic:**

```rust
// Site belongs to user if:
- Site name contains user name ("Yasin Kakande" → "Yasin")
- Site URL contains user name (yasinkakande.wixsite.com → "yasin")
- Custom matching rules (expandable)
```

### **User Experience:**

```
┌─────────────────────────────────────────────┐
│ Your Connected Websites                     │
│                                             │
│ ┌─────────────────────────────────────────┐ │
│ │ Yasin Kakande                    ● Active│ │
│ │ Builder: Wix • Built by QuillSpace      │ │
│ │ Domain: yasinkakande.wixsite.com        │ │
│ │ Visit Website ↗              [Edit] 🎨  │ │
│ └─────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

## 🎨 Client Editing Experience

### **When Yasin clicks "Edit":**

1. **Opens QuillSpace editor** (`/editor/wix/[website_id]`)
2. **Loads Wix content** via API using your credentials
3. **Converts to Puck format** for visual editing
4. **User edits visually** with drag & drop
5. **Saves back to Wix** via API
6. **Publishes to live site** when ready

### **Benefits:**
- ✅ **No setup needed** for clients
- ✅ **Better editing experience** than Wix editor
- ✅ **Unified dashboard** for all their websites
- ✅ **Professional service** delivery

## 🔧 Troubleshooting

### **Website Not Showing Up:**

1. **Check environment variables** are set correctly
2. **Verify Wix API credentials** work
3. **Check site name matching** logic
4. **Look at server logs** for API errors

### **API Connection Test:**

```bash
# Test your Wix credentials
curl -H "Authorization: Bearer YOUR_API_KEY" \
     -H "wix-account-id: YOUR_ACCOUNT_ID" \
     https://www.wixapis.com/site-list/v2/sites
```

### **Debug Matching:**

Add logging to see which sites are found:

```rust
// In get_quillspace_built_websites()
for wix_site in wix_sites {
    println!("Found site: {} for user: {}", wix_site.display_name, user_info.name);
    if self.site_belongs_to_user(&wix_site, &user_info) {
        println!("✅ Match found!");
    }
}
```

## 🚀 Expanding the System

### **Add More Clients:**

The system automatically works for new clients. Just:
1. **Build their website** in your Wix account
2. **Name it appropriately** (e.g., "John Smith Website")
3. **Client logs into QuillSpace** → sees their website

### **Custom Matching Rules:**

Expand the `site_belongs_to_user()` function:

```rust
fn site_belongs_to_user(&self, wix_site: &WixSite, user_info: &UserInfo) -> bool {
    // Add custom logic here
    // - Match by metadata tags
    // - Match by custom fields
    // - Match by project IDs
    // - etc.
}
```

### **Multiple Builders:**

Extend to support other platforms:
- WordPress sites
- Squarespace sites
- Custom QuillSpace sites

## 📊 Expected Results

### **For Yasin:**
- ✅ Sees his website in Connected Websites
- ✅ Can click "Edit" to open QuillSpace editor
- ✅ Can make changes and publish to live site
- ✅ Professional editing experience

### **For QuillSpace:**
- ✅ Seamless client experience
- ✅ No manual setup per client
- ✅ Competitive advantage over other platforms
- ✅ Scalable service delivery model

This creates a **white-label editing experience** where clients get the benefits of QuillSpace's superior editor while their websites remain on your Wix account! 🌟
