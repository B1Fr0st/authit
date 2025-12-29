# Swagger/OpenAPI Documentation

This project includes a complete OpenAPI 3.0 specification that can be used to generate interactive API documentation with Swagger UI.

## Quick Start

### Option 1: Swagger UI (Docker)

The easiest way to view the API documentation:

```bash
docker run -p 8080:8080 -e SWAGGER_JSON=/api/openapi.yaml -v ${PWD}:/api swaggerapi/swagger-ui
```

Then open http://localhost:8080 in your browser.

### Option 2: Swagger Editor

1. Go to https://editor.swagger.io/
2. Click **File** → **Import file**
3. Select `openapi.yaml` from this repository
4. The interactive documentation will load

### Option 3: VS Code Extension

1. Install the "OpenAPI (Swagger) Editor" extension
2. Open `openapi.yaml` in VS Code
3. Right-click and select "OpenAPI: Show Preview"

### Option 4: Redoc

For a cleaner, documentation-focused view:

```bash
docker run -p 8080:80 -e SPEC_URL=/spec/openapi.yaml -v ${PWD}:/usr/share/nginx/html/spec redocly/redoc
```

Then open http://localhost:8080 in your browser.

## OpenAPI File Location

The OpenAPI specification is located at:
```
d:\authit\openapi.yaml
```

## What's Documented

The OpenAPI spec includes complete documentation for:

### Public Endpoints
- `GET /api/v1/public/health` - Health check
- `GET /api/v1/public/auth` - License authorization
- `GET /api/v1/public/product` - Product information

### Private Endpoints (License Management)
- `POST /api/v1/private/license/generator` - Generate new license
- `PUT /api/v1/private/license/add-product` - Add products to license
- `PUT /api/v1/private/license/delete-product` - Remove products from license
- `POST /api/v1/private/license/ban` - Ban a license
- `POST /api/v1/private/license/unban` - Unban a license
- `DELETE /api/v1/private/license/delete` - Delete a license
- `PUT /api/v1/private/license/reset-hwid` - Reset license HWID

### Private Endpoints (HWID Management)
- `POST /api/v1/private/hwid/ban` - Ban a HWID globally
- `POST /api/v1/private/hwid/unban` - Unban a HWID globally

### Private Endpoints (Product Management)
- `PUT /api/v1/private/product/freeze` - Freeze a product
- `PUT /api/v1/private/product/unfreeze` - Unfreeze a product

### Private Endpoints (Data/Monitoring)
- `GET /api/v1/private/data/licenses` - Get all licenses
- `GET /api/v1/private/data/products` - Get all products
- `GET /api/v1/private/data/logins` - Get authorization logs
- `GET /api/v1/private/data/logs` - Get system logs

## Features

The OpenAPI spec includes:

✅ **Complete endpoint documentation** - All endpoints from DESIGN_DOC.md
✅ **Request/Response schemas** - Full data models with examples
✅ **Authentication documentation** - Bearer token auth for private endpoints
✅ **Interactive examples** - Try API calls directly from Swagger UI
✅ **Multiple response examples** - Success and error cases
✅ **Header documentation** - Custom headers for license keys, HWIDs, etc.
✅ **Tags and grouping** - Organized by functionality

## Code Generation

The OpenAPI spec can be used to generate client libraries and server stubs:

### Generate Client (TypeScript)

```bash
npx @openapitools/openapi-generator-cli generate \
  -i openapi.yaml \
  -g typescript-axios \
  -o ./generated/typescript-client
```

### Generate Client (Python)

```bash
docker run --rm -v ${PWD}:/local openapitools/openapi-generator-cli generate \
  -i /local/openapi.yaml \
  -g python \
  -o /local/generated/python-client
```

### Generate Client (Go)

```bash
docker run --rm -v ${PWD}:/local openapitools/openapi-generator-cli generate \
  -i /local/openapi.yaml \
  -g go \
  -o /local/generated/go-client
```

### Available Generators

OpenAPI Generator supports 50+ languages and frameworks:
- JavaScript/TypeScript (axios, fetch, angular, react-query)
- Python (requests, asyncio)
- Java (okhttp, retrofit, jersey)
- C# (.NET, RestSharp)
- Go (native, gin)
- Rust (reqwest)
- PHP (Guzzle)
- Ruby (Faraday)
- And many more...

See full list: https://openapi-generator.tech/docs/generators

## Postman Collection

### Pre-built Collection

A ready-to-use Postman collection is available at `postman_collection.json`:

1. Open Postman
2. Click **Import**
3. Select `postman_collection.json`
4. Configure collection variables:
   - `baseUrl` - Server URL (default: http://localhost:3000)
   - `apiKey` - Your API key for private endpoints
   - `testLicenseKey` - Sample license key for testing
   - `testProductId` - Sample product ID
   - `testHwid` - Sample hardware ID

The collection includes all endpoints with pre-configured requests and examples.

### Import from OpenAPI

You can also import the OpenAPI spec directly:

1. Open Postman
2. Click **Import**
3. Select `openapi.yaml`
4. Postman will create a collection with all endpoints

## Validation

Validate the OpenAPI spec:

```bash
# Using Swagger CLI
npx @apidevtools/swagger-cli validate openapi.yaml

# Using OpenAPI Generator
docker run --rm -v ${PWD}:/local openapitools/openapi-generator-cli validate \
  -i /local/openapi.yaml
```

## Updating the Spec

When adding new endpoints:

1. Update `openapi.yaml` with the new endpoint definition
2. Add request/response schemas if needed
3. Add examples for all response types
4. Validate the spec
5. Regenerate client libraries if using code generation

## Integration with CI/CD

### Validate on Push

```yaml
# .github/workflows/openapi-validation.yml
name: Validate OpenAPI
on: [push]
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Validate OpenAPI spec
        run: |
          npx @apidevtools/swagger-cli validate openapi.yaml
```

### Auto-generate Documentation

You can automatically publish Swagger UI documentation on every commit using GitHub Pages or similar services.

## Servers

The spec defines two servers:

- **Development:** http://localhost:3000
- **Production:** https://api.example.com (update this for your domain)

You can switch between them in Swagger UI.

## Tips

1. **Try It Out** - Swagger UI has a "Try it out" button to test endpoints directly
2. **Authentication** - Click "Authorize" in Swagger UI to set your API key once for all private endpoints
3. **Examples** - Each endpoint has multiple examples showing different response types
4. **Schemas** - Click on schema names to see their full definitions
5. **Download** - You can download the spec from Swagger UI for offline use

## Resources

- [OpenAPI Specification](https://swagger.io/specification/)
- [Swagger UI](https://swagger.io/tools/swagger-ui/)
- [Swagger Editor](https://editor.swagger.io/)
- [OpenAPI Generator](https://openapi-generator.tech/)
- [Redoc](https://redocly.com/redoc/)
