# Authit License Server

A high-performance license management and authorization server built with Rust.

## Features

- ✅ License key generation and management
- ✅ Hardware ID (HWID) binding and validation
- ✅ Product-based time allocations
- ✅ Authorization logging and monitoring
- ✅ Product freeze/unfreeze capabilities
- ✅ Bearer token authentication for admin endpoints
- ✅ PostgreSQL database backend
- ✅ Docker deployment ready
- ✅ Complete OpenAPI 3.0 specification

## Quick Start

### Run with Docker

```bash
# Set your API key
export API_KEY="your-secure-api-key"

# Start the stack
docker compose up --build
```

The server will be available at http://localhost:3000

### Run Locally

```bash
# Set environment variables
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/authit"
export API_KEY="your-secure-api-key"

# Run the server
cargo run --release
```

## Documentation

- **[API Documentation](API.md)** - Complete API reference with examples
- **[OpenAPI Spec](openapi.yaml)** - Machine-readable API specification
- **[Swagger UI Guide](SWAGGER.md)** - Interactive API documentation
- **[Design Document](DESIGN_DOC.md)** - Architecture and design decisions
- **[Docker Guide](DOCKER.md)** - Docker deployment instructions
- **[Authorization Guide](AUTHORIZATION.md)** - Authentication system details

## Project Structure

Types all should live in their reasonably respective files:
-   Request/Response types in `/types/requests.rs`
-   Monitoring/Data types in `/types/data.rs`
-   Core types in `/types/core.rs` (License, Product, internal types)

Handlers are broken up according to DESIGN_DOC.md, and can be read such as the following:
Assuming that `/api/v1/` is the root, any given endpoint can be found in its respective mod.rs/NAME.rs file.
-   `/private/license/generator` would be in `/handlers/private/license.rs`
-   `/public/auth` would be in `/handlers/public/mod.rs`

## API Endpoints

### Public Endpoints (No Authentication)

- `GET /api/v1/public/health` - Health check
- `GET /api/v1/public/auth` - License authorization (headers: X-License-Key, X-Product-ID, X-HWID)
- `GET /api/v1/public/product` - Product information (headers: X-License-Key, X-Product-ID)

### Private Endpoints (Require API Key)

All private endpoints require `Authorization: Bearer <api-key>` header.

- `POST /api/v1/private/license/generator` - Generate new license
- See [API.md](API.md) or [openapi.yaml](openapi.yaml) for complete endpoint list

## Environment Variables

```bash
# Database connection
DATABASE_URL="postgres://postgres:postgres@localhost:5432/authit"

# API key for private endpoints (REQUIRED for production)
API_KEY="your-secure-api-key-here"
```

See [.env.example](.env.example) for more details.

## Interactive API Documentation

View the complete API documentation with Swagger UI:

```bash
docker run -p 8080:8080 -e SWAGGER_JSON=/api/openapi.yaml -v ${PWD}:/api swaggerapi/swagger-ui
```

Then open http://localhost:8080 in your browser.

See [SWAGGER.md](SWAGGER.md) for more options.

## Development

### Prerequisites

- Rust 1.75+
- PostgreSQL 16+
- Docker (optional)

### Building

```bash
cargo build --release
```

### Testing

```bash
cargo test
```

## Security

- All sensitive data (license keys, HWIDs) transmitted via HTTP headers, not query parameters
- Bearer token authentication for administrative endpoints
- API keys configured via environment variables
- Comprehensive authorization logging
- **Always use HTTPS in production**

See [AUTHORIZATION.md](AUTHORIZATION.md) for security best practices.

## License

MIT