# Build stage
FROM rust:1.82-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev pkgconfig openssl-dev

# Create app directory
WORKDIR /usr/src/erp

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build the application in release mode
RUN cargo build --release --bin erp-server

# Runtime stage
FROM alpine:3.20

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libgcc \
    openssl

# Create non-root user
RUN addgroup -S erp && adduser -S erp -G erp

# Copy the binary from builder
COPY --from=builder /usr/src/erp/target/release/erp-server /usr/local/bin/erp-server

# Copy migration files
COPY migrations /app/migrations

# Set ownership
RUN chown -R erp:erp /app

# Switch to non-root user
USER erp

# Set working directory
WORKDIR /app

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

# Run the application
CMD ["erp-server"]