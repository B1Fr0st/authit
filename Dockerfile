FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies including postgresql-client for migrations
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    postgresql-client \
    dos2unix \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built binary from builder
COPY --from=builder /app/target/release/authit /app/authit

# Copy migrations directory and entrypoint script
COPY migrations /app/migrations
COPY docker-entrypoint.sh /app/docker-entrypoint.sh

# Convert line endings and make entrypoint script executable
RUN dos2unix /app/docker-entrypoint.sh && \
    chmod +x /app/docker-entrypoint.sh

# Expose the application port
EXPOSE 5593

# Set entrypoint to run migrations before starting the app
ENTRYPOINT ["/app/docker-entrypoint.sh"]
CMD ["/app/authit"]
