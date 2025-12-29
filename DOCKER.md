# Docker Setup

This guide explains how to run the authit license server using Docker.

## Prerequisites

- Docker installed and running
- Docker Compose installed

## Quick Start

1. **Build and start the services:**
   ```bash
   docker compose up --build
   ```

2. **Run in detached mode:**
   ```bash
   docker compose up -d --build
   ```

3. **Stop the services:**
   ```bash
   docker compose down
   ```

4. **Stop and remove volumes (clears database):**
   ```bash
   docker compose down -v
   ```

## Services

### authit (Application)
- **Port:** 3000
- **Endpoints:**
  - `GET http://localhost:3000/api/v1/public/health` - Health check
  - `GET http://localhost:3000/api/v1/public/auth` - License authorization
  - `GET http://localhost:3000/api/v1/public/product` - Product information

### postgres (Database)
- **Port:** 5432 (exposed on host)
- **Database:** authit
- **User:** postgres
- **Password:** postgres

## Environment Variables

The application uses the following environment variable:

- `DATABASE_URL` - PostgreSQL connection string (default: `postgres://postgres:postgres@postgres:5432/authit`)

You can override this in the `docker-compose.yml` file if needed.

## Logs

View application logs:
```bash
docker compose logs authit
```

View database logs:
```bash
docker compose logs postgres
```

Follow logs in real-time:
```bash
docker compose logs -f
```

## Development

For development, you can run just the PostgreSQL database:
```bash
docker compose up postgres
```

Then run the Rust application locally:
```bash
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/authit
cargo run
```

## Database Access

Connect to PostgreSQL directly:
```bash
docker compose exec postgres psql -U postgres -d authit
```

Or from your host machine:
```bash
psql -h localhost -U postgres -d authit
```

## Troubleshooting

### Port already in use
If port 3000 or 5432 is already in use, modify the port mappings in `docker-compose.yml`:
```yaml
ports:
  - "8080:3000"  # Use port 8080 on host instead of 3000
```

### Database connection issues
Ensure the PostgreSQL service is healthy before the application starts. The `depends_on` with `service_healthy` condition ensures proper startup order.

### Rebuild after code changes
```bash
docker compose up --build
```
