# Environment Variables Setup Guide

This guide covers all environment variables needed for QuillSpace's consultation system.

## 🔑 Required Variables

### Calendly Integration

| Variable | Required | Description | How to Get |
|----------|----------|-------------|------------|
| `CALENDLY_API_TOKEN` | ✅ Yes | Personal access token for Calendly API | [Calendly Developer Settings](https://calendly.com/integrations/api_webhooks) |
| `CALENDLY_ORGANIZATION_URI` | ✅ Yes | Your organization URI from Calendly | API call: `GET /users/me` |
| `CALENDLY_WEBHOOK_SECRET` | ✅ Yes | Secret for webhook signature verification | Generate: `openssl rand -hex 32` |
| `CALENDLY_WEBHOOK_URL` | ❌ No | Public webhook endpoint URL | Default: `https://api.quillspace.io/webhooks/calendly` |
| `CALENDLY_REDIRECT_URL` | ❌ No | Post-booking redirect URL | Default: `https://app.quillspace.io/consultation-booked` |

### Email Service (Choose One)

#### Option A: SendGrid (Recommended)
| Variable | Required | Description |
|----------|----------|-------------|
| `SENDGRID_API_KEY` | ✅ Yes | SendGrid API key with full access |
| `SENDGRID_FROM_EMAIL` | ✅ Yes | Verified sender email address |
| `SENDGRID_FROM_NAME` | ❌ No | Display name for emails |

#### Option B: AWS SES
| Variable | Required | Description |
|----------|----------|-------------|
| `AWS_ACCESS_KEY_ID` | ✅ Yes | AWS access key for SES |
| `AWS_SECRET_ACCESS_KEY` | ✅ Yes | AWS secret key |
| `AWS_REGION` | ✅ Yes | AWS region (e.g., us-east-1) |
| `AWS_SES_FROM_EMAIL` | ✅ Yes | Verified SES email address |

## 📋 Step-by-Step Setup

### 1. Calendly Setup

#### Get API Token:
1. Go to [Calendly Developer Settings](https://calendly.com/integrations/api_webhooks)
2. Click "Create Personal Access Token"
3. Name it "QuillSpace Integration"
4. Copy the token

#### Get Organization URI:
```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
     https://api.calendly.com/users/me
```
Look for `current_organization` in the response.

#### Generate Webhook Secret:
```bash
openssl rand -hex 32
```

### 2. Email Service Setup

#### SendGrid (Recommended):
1. Sign up at [SendGrid](https://sendgrid.com)
2. Go to Settings → API Keys
3. Create new API key with "Full Access"
4. Go to Settings → Sender Authentication
5. Verify your domain or single sender email

#### AWS SES:
1. Go to AWS Console → Simple Email Service
2. Verify your sending domain in "Verified identities"
3. Create IAM user with `AmazonSESFullAccess` policy
4. Generate access key and secret

### 3. Environment File

Create `.env.production`:

```bash
# Calendly Integration
CALENDLY_API_TOKEN=eyJhbGciOiJIUzI1NiJ9...
CALENDLY_ORGANIZATION_URI=https://api.calendly.com/organizations/AAAAAAAAAAAAAAAA
CALENDLY_WEBHOOK_SECRET=a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6
CALENDLY_WEBHOOK_URL=https://api.quillspace.io/webhooks/calendly
CALENDLY_REDIRECT_URL=https://app.quillspace.io/consultation-booked

# Email Service (SendGrid)
SENDGRID_API_KEY=SG.abc123...
SENDGRID_FROM_EMAIL=hello@quillspace.io
SENDGRID_FROM_NAME=QuillSpace Team

# Database (existing)
DATABASE_URL=postgresql://user:pass@localhost:5432/quillspace
```

### 4. Run Setup Script

```bash
# Make script executable
chmod +x scripts/setup-calendly.sh

# Run setup (after setting environment variables)
./scripts/setup-calendly.sh
```

## 🧪 Testing Your Setup

### 1. Test Calendly API Connection:
```bash
curl -H "Authorization: Bearer $CALENDLY_API_TOKEN" \
     https://api.calendly.com/users/me
```

### 2. Test Webhook Endpoint:
```bash
curl -X POST \
     -H "Content-Type: application/json" \
     -d '{"event": "test"}' \
     $CALENDLY_WEBHOOK_URL
```

### 3. Test Email Service:
- SendGrid: Use their API test endpoint
- AWS SES: Send test email through console

### 4. End-to-End Test:
1. Make a test booking on your Calendly page
2. Check webhook delivery in logs
3. Verify user receives confirmation email
4. Test project brief form access

## 🚨 Security Notes

### Webhook Security:
- Always verify webhook signatures using `CALENDLY_WEBHOOK_SECRET`
- Use HTTPS for all webhook URLs
- Validate webhook payload structure

### API Keys:
- Never commit API keys to version control
- Use different keys for development/production
- Rotate keys regularly
- Restrict API key permissions to minimum required

### Environment Variables:
- Use secure secret management in production
- Consider using services like AWS Secrets Manager
- Encrypt sensitive values at rest

## 🔧 Troubleshooting

### Common Issues:

#### "Unauthorized" from Calendly API:
- Check your API token is correct
- Ensure token has required permissions
- Verify organization URI format

#### Webhook not receiving events:
- Check webhook URL is publicly accessible
- Verify SSL certificate is valid
- Check Calendly webhook subscription status

#### Email delivery issues:
- Verify sender email/domain is authenticated
- Check API key permissions
- Review email service logs

#### Database connection errors:
- Verify DATABASE_URL format
- Check database server is running
- Ensure user has required permissions

### Debug Commands:

```bash
# Test Calendly connection
curl -v -H "Authorization: Bearer $CALENDLY_API_TOKEN" \
     https://api.calendly.com/users/me

# List webhook subscriptions
curl -H "Authorization: Bearer $CALENDLY_API_TOKEN" \
     "https://api.calendly.com/webhook_subscriptions?organization=$CALENDLY_ORGANIZATION_URI"

# Test webhook signature verification
echo "payload" | openssl dgst -sha256 -hmac "$CALENDLY_WEBHOOK_SECRET"
```

## 📞 Support

If you encounter issues:

1. Check the troubleshooting section above
2. Review application logs for error details
3. Test individual components (API, webhook, email)
4. Verify all environment variables are set correctly

For Calendly-specific issues, refer to their [API documentation](https://developer.calendly.com/api-docs/).
