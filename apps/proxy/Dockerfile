FROM rust:1.75 as builder

WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim
COPY --from=builder /usr/local/cargo/bin/ai-analytics-proxy /usr/local/bin/
CMD ["ai-analytics-proxy", "--help"]
