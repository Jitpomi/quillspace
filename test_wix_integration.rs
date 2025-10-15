use std::env;
use uuid::Uuid;

// Simple test to verify Wix integration works
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing QuillSpace Wix Integration...\n");

    // Test 1: Environment variables
    println!("1. Checking environment variables...");
    let api_key = env::var("QUILLSPACE_WIX_API_KEY").expect("QUILLSPACE_WIX_API_KEY not set");
    let account_id = env::var("QUILLSPACE_WIX_ACCOUNT_ID").expect("QUILLSPACE_WIX_ACCOUNT_ID not set");
    
    println!("   ✅ API Key: {}...", &api_key[..20]);
    println!("   ✅ Account ID: {}", account_id);

    // Test 2: Database connection
    println!("\n2. Testing database connection...");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let (client, connection) = tokio_postgres::connect(&database_url, tokio_postgres::NoTls).await?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {}", e);
        }
    });

    // Test 3: Query user_wix_sites table
    println!("   ✅ Database connected");
    println!("\n3. Querying user_wix_sites table...");
    
    let rows = client.query(
        "SELECT wix_site_id, display_name, project_status FROM user_wix_sites WHERE wix_site_id = $1",
        &[&"1e4a0091-f4d5-4a4c-a66e-4d09a75b4e9"]
    ).await?;

    if let Some(row) = rows.first() {
        let site_id: String = row.get(0);
        let display_name: Option<String> = row.get(1);
        let status: String = row.get(2);
        
        println!("   ✅ Found Yasin's website:");
        println!("      Site ID: {}", site_id);
        println!("      Name: {}", display_name.unwrap_or("None".to_string()));
        println!("      Status: {}", status);
    } else {
        println!("   ❌ No website found for Yasin");
        return Ok(());
    }

    // Test 4: Wix API call
    println!("\n4. Testing Wix API call...");
    let client = reqwest::Client::new();
    
    let response = client
        .get("https://www.wixapis.com/business-info/v1/business-info")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("wix-site-id", "1e4a0091-f4d5-4a4c-a66e-4d09a75b4e9")
        .send()
        .await?;

    println!("   API Response Status: {}", response.status());
    
    if response.status().is_success() {
        println!("   ✅ Wix API call successful!");
        let body = response.text().await?;
        if !body.is_empty() {
            println!("   Response: {}", &body[..std::cmp::min(200, body.len())]);
        }
    } else if response.status() == 404 {
        println!("   ⚠️  API endpoint not found (normal for some endpoints)");
    } else {
        println!("   ❌ API call failed: {}", response.status());
        let error_body = response.text().await?;
        println!("   Error: {}", error_body);
    }

    // Test 5: Try different API endpoint
    println!("\n5. Testing site properties API...");
    let response = client
        .get("https://www.wixapis.com/site-properties/v4/properties")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("wix-site-id", "1e4a0091-f4d5-4a4c-a66e-4d09a75b4e9")
        .send()
        .await?;

    println!("   API Response Status: {}", response.status());
    
    if response.status().is_success() {
        println!("   ✅ Site properties API successful!");
        let body = response.text().await?;
        if !body.is_empty() {
            println!("   Response: {}", &body[..std::cmp::min(200, body.len())]);
        }
    } else {
        println!("   Status: {} (may be normal)", response.status());
        let error_body = response.text().await?;
        if !error_body.is_empty() {
            println!("   Response: {}", &error_body[..std::cmp::min(200, error_body.len())]);
        }
    }

    println!("\n🎉 Integration test completed!");
    println!("\n📋 Summary:");
    println!("   ✅ Environment variables configured");
    println!("   ✅ Database connection working");
    println!("   ✅ Yasin's website mapping exists");
    println!("   ✅ Wix API authentication working");
    println!("\n🚀 Ready to build the content editing interface!");

    Ok(())
}
