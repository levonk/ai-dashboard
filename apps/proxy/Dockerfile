FROM rust:1.75 as builder

WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim

# Install runtime dependencies including curl for health checks
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -u 1000 analytics && \
    chown -R analytics:analytics /home/analytics

COPY --from=builder /usr/local/cargo/bin/ai-analytics-proxy /usr/local/bin/

USER analytics
WORKDIR /home/analytics

# Expose the default port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

# Default command - can be overridden
CMD ["ai-analytics-proxy", "serve", "--port", "8080"]
