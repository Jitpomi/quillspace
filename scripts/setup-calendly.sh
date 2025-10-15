#!/bin/bash

# QuillSpace Calendly Integration Setup Script
# Run this after setting your environment variables

set -e

echo "🚀 Setting up Calendly integration for QuillSpace..."

# Check required environment variables
if [ -z "$CALENDLY_API_TOKEN" ]; then
    echo "❌ CALENDLY_API_TOKEN is required"
    exit 1
fi

if [ -z "$CALENDLY_ORGANIZATION_URI" ]; then
    echo "❌ CALENDLY_ORGANIZATION_URI is required"
    exit 1
fi

if [ -z "$CALENDLY_WEBHOOK_SECRET" ]; then
    echo "❌ CALENDLY_WEBHOOK_SECRET is required"
    exit 1
fi

# Set defaults
WEBHOOK_URL=${CALENDLY_WEBHOOK_URL:-"https://api.quillspace.io/webhooks/calendly"}
REDIRECT_URL=${CALENDLY_REDIRECT_URL:-"https://app.quillspace.io/consultation-booked"}

echo "📋 Configuration:"
echo "  Webhook URL: $WEBHOOK_URL"
echo "  Redirect URL: $REDIRECT_URL"
echo "  Organization: $CALENDLY_ORGANIZATION_URI"

# 1. Get current user info
echo "👤 Getting user information..."
USER_RESPONSE=$(curl -s -H "Authorization: Bearer $CALENDLY_API_TOKEN" \
    https://api.calendly.com/users/me)

if echo "$USER_RESPONSE" | grep -q "error"; then
    echo "❌ Failed to authenticate with Calendly API"
    echo "$USER_RESPONSE"
    exit 1
fi

echo "✅ Successfully authenticated with Calendly"

# 2. List existing webhooks
echo "🔍 Checking existing webhooks..."
WEBHOOKS_RESPONSE=$(curl -s -H "Authorization: Bearer $CALENDLY_API_TOKEN" \
    "https://api.calendly.com/webhook_subscriptions?organization=$CALENDLY_ORGANIZATION_URI")

echo "📝 Existing webhooks:"
echo "$WEBHOOKS_RESPONSE" | jq '.collection[] | {uri: .uri, callback_url: .callback_url, events: .events}' 2>/dev/null || echo "No existing webhooks found"

# 3. Create new webhook subscription
echo "🔗 Creating webhook subscription..."
WEBHOOK_DATA='{
    "url": "'$WEBHOOK_URL'",
    "events": [
        "invitee.created",
        "invitee.canceled"
    ],
    "organization": "'$CALENDLY_ORGANIZATION_URI'",
    "scope": "organization"
}'

WEBHOOK_RESPONSE=$(curl -s -X POST \
    -H "Authorization: Bearer $CALENDLY_API_TOKEN" \
    -H "Content-Type: application/json" \
    -d "$WEBHOOK_DATA" \
    https://api.calendly.com/webhook_subscriptions)

if echo "$WEBHOOK_RESPONSE" | grep -q "error"; then
    echo "⚠️  Webhook creation response:"
    echo "$WEBHOOK_RESPONSE"
else
    echo "✅ Webhook subscription created successfully!"
    echo "$WEBHOOK_RESPONSE" | jq '.resource' 2>/dev/null || echo "$WEBHOOK_RESPONSE"
fi

# 4. List event types
echo "📅 Getting your event types..."
EVENT_TYPES_RESPONSE=$(curl -s -H "Authorization: Bearer $CALENDLY_API_TOKEN" \
    "https://api.calendly.com/event_types?organization=$CALENDLY_ORGANIZATION_URI")

echo "📋 Your event types:"
echo "$EVENT_TYPES_RESPONSE" | jq '.collection[] | {name: .name, uri: .uri, booking_url: .scheduling_url}' 2>/dev/null || echo "Could not parse event types"

# 5. Update event types with redirect URL (optional)
if [ ! -z "$CALENDLY_EVENT_TYPE_URIS" ]; then
    echo "🔄 Updating event types with redirect URL..."
    IFS=',' read -ra EVENT_URIS <<< "$CALENDLY_EVENT_TYPE_URIS"
    for uri in "${EVENT_URIS[@]}"; do
        echo "  Updating: $uri"
        # Note: This might require additional API permissions
        # curl -s -X PATCH \
        #     -H "Authorization: Bearer $CALENDLY_API_TOKEN" \
        #     -H "Content-Type: application/json" \
        #     -d '{"redirect_url": "'$REDIRECT_URL'?event={{event_uuid}}&invitee={{invitee_uuid}}"}' \
        #     "$uri"
    done
fi

echo ""
echo "🎉 Calendly integration setup complete!"
echo ""
echo "📝 Next steps:"
echo "1. Test your webhook endpoint: $WEBHOOK_URL"
echo "2. Make a test booking to verify the integration"
echo "3. Check your application logs for webhook events"
echo ""
echo "🔧 Webhook verification:"
echo "   Use CALENDLY_WEBHOOK_SECRET to verify webhook signatures"
echo "   Secret: $CALENDLY_WEBHOOK_SECRET"
echo ""
echo "📱 Badge widget configuration:"
echo "   URL: https://calendly.com/dev-jitpomi/30min"
echo "   Text: 'Get Your Author Website ✨'"
echo "   Color: #9caf88"
