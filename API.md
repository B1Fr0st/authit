# API Documentation

## Authentication & Security

All sensitive endpoints use **HTTP headers** instead of query parameters for enhanced security. This prevents license keys and other sensitive data from appearing in server logs, browser history, or proxy logs.

## Public Endpoints

Base URL: `/api/v1/public`

### Health Check

**Endpoint:** `GET /health`

**Description:** Simple health check endpoint to verify the server is running.

**Response:**
- Status: `200 OK`

**Example:**
```bash
curl http://localhost:3000/api/v1/public/health
```

---

### License Authorization

**Endpoint:** `GET /auth`

**Description:** Authorizes a license key with a hardware ID (HWID) for a specific product. On first use, the HWID is bound to the license. Subsequent requests must use the same HWID.

**Headers:**
- `X-License-Key`: The license key to authorize
- `X-Product-ID`: The product ID to check
- `X-HWID`: Hardware identifier for the client machine

**Response:**
```json
"Ok" | "InvalidLicense" | "HWIDMismatch" | "LicenseExpired" | "LicenseFrozen" | "MissingHeaders"
```

**Response Codes:**
- `Ok` - Authorization successful
- `InvalidLicense` - License key doesn't exist or doesn't have access to the requested product
- `HWIDMismatch` - The provided HWID doesn't match the one bound to the license
- `LicenseExpired` - The license time has expired for this product
- `LicenseFrozen` - The product is currently frozen/suspended
- `MissingHeaders` - Required headers are missing from the request

**Example:**
```bash
curl -X GET http://localhost:3000/api/v1/public/auth \
  -H "X-License-Key: ABC123-DEF456-GHI789" \
  -H "X-Product-ID: premium-software-v1" \
  -H "X-HWID: machine-12345"
```

**Behavior:**
1. First request with a new license binds the HWID to the license
2. Checks if the product exists in the license
3. Verifies the product is not frozen
4. Validates the license hasn't expired based on time elapsed since activation
5. Logs the authorization attempt for monitoring

---

### Product Information

**Endpoint:** `GET /product`

**Description:** Retrieves information about a specific product subscription on a license, including time remaining and activation status.

**Headers:**
- `X-License-Key`: The license key to query
- `X-Product-ID`: The product ID to retrieve information for

**Response:**
```json
{
  "Ok": {
    "product": "premium-software-v1",
    "time": 2592000,
    "started_at": 1703001234
  }
}
```

Or error responses:
```json
"InvalidLicense" | "InvalidProduct" | "MissingHeaders"
```

**Response Fields (on success):**
- `product` - The product identifier
- `time` - Total subscription time in seconds
- `started_at` - Unix timestamp when the subscription started

**Example:**
```bash
curl -X GET http://localhost:3000/api/v1/public/product \
  -H "X-License-Key: ABC123-DEF456-GHI789" \
  -H "X-Product-ID: premium-software-v1"
```

---

## Private Endpoints

Base URL: `/api/v1/private`

**Authentication Required:** All private endpoints require a valid API key passed in the `Authorization` header using the Bearer token scheme.

**Header Format:**
```
Authorization: Bearer <your-api-key>
```

**Setting the API Key:**
The API key is configured via the `API_KEY` environment variable. If not set, a default insecure key is used (not recommended for production).

```bash
export API_KEY="your-secret-api-key-here"
```

**Unauthorized Response:**
If authentication fails, you'll receive a `401 Unauthorized` status with:
```json
{
  "error": "Unauthorized - Valid API key required"
}
```

---

### License Management

#### Generate License

**Endpoint:** `POST /license/generator`

**Description:** Generates a new license key with the specified products and time allocations. The license key is automatically generated in the format `XXXXX-XXXXX-XXXXX`.

**Headers:**
- `Authorization: Bearer <api-key>` - Required for authentication
- `Content-Type: application/json`

**Request Body:**
```json
{
  "products": {
    "product-id-1": 2592000,
    "product-id-2": 7776000
  }
}
```

**Request Fields:**
- `products` - Map of product IDs to time in seconds

**Response:**
```json
{
  "Ok": "ABCD1-EFGH2-IJKL3"
}
```

Or error responses:
```json
"OneOrMoreInvalidProduct" | "FailedToGenerateValidLicense"
```

**Response Types:**
- `Ok(string)` - License generated successfully, returns the license key
- `OneOrMoreInvalidProduct` - One or more product IDs don't exist in the system
- `FailedToGenerateValidLicense` - Failed to generate a unique license key after 10 attempts

**Example:**
```bash
curl -X POST http://localhost:3000/api/v1/private/license/generator \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "products": {
      "premium-software-v1": 2592000,
      "addon-pack": 2592000
    }
  }'
```

**Behavior:**
1. Validates all product IDs exist in the database
2. Generates a random, unique license key
3. Sets the current timestamp as `started_at` for all products
4. HWID is initially empty and will be bound on first `/auth` call
5. Returns the generated license key

---

### Additional Private Endpoints

The following endpoints are planned but not yet implemented:

#### License Management
- `PUT /license/add-product` - Add products to an existing license
- `PUT /license/delete-product` - Remove products from a license
- `POST /license/ban` - Ban a license (but not its HWID)
- `POST /license/unban` - Unban a license
- `DELETE /license/delete` - Delete a license entirely
- `PUT /license/reset-hwid` - Reset the HWID binding for a license

#### HWID Management
- `POST /hwid/ban` - Ban a HWID across all licenses
- `POST /hwid/unban` - Unban a HWID

#### Product Management
- `PUT /product/freeze` - Freeze a product (prevents authorization)
- `PUT /product/unfreeze` - Unfreeze a product

#### Data/Monitoring
- `GET /data/licenses` - Retrieve all licenses with their sessions
- `GET /data/products` - Retrieve all products in the system
- `GET /data/logins` - Retrieve authorization attempt logs
- `GET /data/logs` - Retrieve system logs

---

## Error Handling

All endpoints return JSON responses. HTTP status codes are currently `200 OK` for all responses, with the actual status indicated in the JSON response body.

**Common Error Responses:**
- `MissingHeaders` - One or more required headers are missing
- `InvalidLicense` - License key not found or invalid
- `InvalidProduct` - Product not found or not associated with license

---

## Data Types

### Time Values
All time values are represented as **Unix timestamps** (seconds since epoch) or duration in seconds.

### License Keys
License keys are arbitrary strings. The format is defined by the license generation system.

### Product IDs
Product identifiers are strings that uniquely identify each product/service.

### HWID (Hardware ID)
A unique identifier for the client machine. The format is determined by the client application.

---

## Best Practices

1. **Always use HTTPS** in production to encrypt header data in transit
2. **Cache authorization responses** on the client side to minimize API calls
3. **Implement retry logic** with exponential backoff for failed requests
4. **Store sensitive data securely** on the client side (license keys, etc.)
5. **Handle all response types** including error cases gracefully
6. **Log authorization failures** on the client side for debugging

---

## Rate Limiting

Currently, there are no rate limits enforced. However, clients should implement reasonable request throttling to avoid overwhelming the server.

---

## Monitoring & Logging

All `/auth` endpoint requests are logged to the database with:
- License key
- HWID
- Timestamp
- Response status

This data can be retrieved via the `/api/v1/private/data/logins` endpoint (requires authorization).
