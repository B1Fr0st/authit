# Authorization System

This document describes the authorization system for private API endpoints.

## Overview

The authit server uses a **Bearer token authentication** scheme for all private API endpoints. This provides a simple yet effective way to secure administrative operations like license generation and management.

## How It Works

### 1. API Key Configuration

The API key is configured via the `API_KEY` environment variable:

```bash
export API_KEY="your-super-secret-key-here"
```

**Important Security Notes:**
- Never commit the API key to version control
- Use a long, randomly generated string (minimum 32 characters recommended)
- Rotate the API key periodically
- Use different keys for development, staging, and production environments

### 2. Authentication Flow

```
Client Request
    ↓
    ├─ Public Endpoint (e.g., /auth, /product)
    │   └─ No authentication required
    │
    └─ Private Endpoint (e.g., /license/generator)
        ↓
        Extract Authorization header
        ↓
        Check format: "Bearer <token>"
        ↓
        Compare token with API_KEY env var
        ↓
        ├─ Match → Process request
        └─ No match → Return 401 Unauthorized
```

### 3. Making Authenticated Requests

Include the Authorization header in all requests to private endpoints:

```bash
curl -X POST http://localhost:3000/api/v1/private/license/generator \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '{"products": {"premium-v1": 2592000}}'
```

### 4. Error Responses

**Missing Authorization Header:**
```
HTTP/1.1 401 Unauthorized
```

**Invalid/Incorrect API Key:**
```
HTTP/1.1 401 Unauthorized
```

**Malformed Authorization Header:**
```
HTTP/1.1 401 Unauthorized
```

## Implementation Details

### Authorization Module

Location: `src/handlers/private/auth.rs`

**Key Functions:**
- `extract_and_validate_auth(req: &Request) -> Result<()>`
  - Extracts the Authorization header
  - Validates the Bearer token format
  - Compares against the configured API key
  - Returns `401 Unauthorized` on failure

### Usage in Handlers

All private endpoint handlers must call the auth function:

```rust
#[handler]
pub async fn my_private_endpoint(
    req: &Request,
    Data(pool): Data<&DbPool>,
) -> Result<impl IntoResponse> {
    // Validate authorization FIRST
    extract_and_validate_auth(req)?;

    // Then process the request
    // ...
}
```

## Docker & Environment Configuration

### Docker Compose

The `docker-compose.yml` automatically passes the `API_KEY` environment variable:

```yaml
environment:
  API_KEY: ${API_KEY:-default-insecure-key}
```

Set it before running Docker:

```bash
export API_KEY="production-secret-key"
docker compose up
```

### Environment File

Copy `.env.example` to `.env` and configure:

```bash
cp .env.example .env
# Edit .env with your values
```

**Never commit `.env` to version control!** It's already in `.gitignore`.

## Security Recommendations

### Production Deployment

1. **Use Strong Keys:**
   ```bash
   # Generate a secure random key
   openssl rand -base64 32
   ```

2. **Enable HTTPS:**
   - All requests should use HTTPS to prevent API key interception
   - The Bearer token is sent in plaintext in headers
   - Without HTTPS, the key can be stolen via man-in-the-middle attacks

3. **Rotate Keys Regularly:**
   - Change the API key every 90 days
   - Immediately rotate if compromised

4. **Use Secrets Management:**
   - For production, use a secrets manager (AWS Secrets Manager, HashiCorp Vault, etc.)
   - Don't hardcode keys in deployment scripts

5. **Monitor Access:**
   - Log all private endpoint access
   - Alert on unusual patterns
   - Track failed authentication attempts

### Key Storage

**DO:**
- ✅ Store in environment variables
- ✅ Use secrets management systems
- ✅ Restrict access to key files
- ✅ Use different keys per environment

**DON'T:**
- ❌ Commit keys to git
- ❌ Share keys in chat/email
- ❌ Use weak/simple keys
- ❌ Reuse keys across services

## Testing

### Development

For development, the server uses a default key if `API_KEY` is not set:

```
WARNING: Using default API key. Set API_KEY environment variable for production!
```

**Default key:** `default-insecure-key`

This is ONLY acceptable for local development. Never use in production!

### Testing Authenticated Endpoints

```bash
# Set API key for testing
export API_KEY="test-key-123"

# Start server
cargo run

# Test endpoint
curl -X POST http://localhost:3000/api/v1/private/license/generator \
  -H "Authorization: Bearer test-key-123" \
  -H "Content-Type: application/json" \
  -d '{"products": {"test-product": 86400}}'
```

### Test Unauthorized Access

```bash
# Missing header
curl -X POST http://localhost:3000/api/v1/private/license/generator

# Invalid key
curl -X POST http://localhost:3000/api/v1/private/license/generator \
  -H "Authorization: Bearer wrong-key"
```

Both should return `401 Unauthorized`.

## Future Enhancements

Potential improvements to the authorization system:

1. **Multiple API Keys** - Support for different keys with different permissions
2. **API Key Management Endpoints** - Create/revoke keys via API
3. **Rate Limiting** - Prevent brute force attacks
4. **Audit Logging** - Detailed logs of who accessed what and when
5. **Key Expiration** - Automatic expiration of API keys
6. **Scoped Permissions** - Different keys for different operations
7. **JWT Tokens** - Short-lived session tokens instead of static keys

## Troubleshooting

**Server warns about default API key:**
- Set the `API_KEY` environment variable before starting the server

**401 Unauthorized errors:**
- Check that the Authorization header is included
- Verify the Bearer token format: `Bearer <key>`
- Confirm the key matches the server's `API_KEY` value
- Ensure no extra whitespace in the header

**Key not being recognized:**
- Restart the server after changing `API_KEY`
- Check for typos in the environment variable name
- Verify the key doesn't have quotes or special characters that need escaping
