# Builder stage
FROM rust:1.81.0 AS builder

WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release
ENTRYPOINT [ "./target/release/zero2prod" ]

# Runtime stage
FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT=production
ENTRYPOINT [ "./zero2prod" ]