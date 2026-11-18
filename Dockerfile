# Build stage
FROM rust:bookworm AS builder
WORKDIR /app

# Cache dependency builds
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install ca-certificates, curl, and archive extraction utilities
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    tar \
    gzip \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# URL specified during the build environment to download the card pics archive
ARG PICS_URL
ENV CARD_IMAGES_PATH=/app/pics

# Download and extract the .tar.gz archive containing the card pictures
RUN mkdir -p /app/pics \
    && if [ -n "${PICS_URL}" ]; then \
         curl -fsSL "${PICS_URL}" -o /tmp/pics.tar.gz \
         && tar -xzf /tmp/pics.tar.gz -C /app/pics \
         && rm -f /tmp/pics.tar.gz; \
       else \
         echo "Notice: PICS_URL not specified at build time."; \
       fi

# Copy the compiled binary from builder stage
COPY --from=builder /app/target/release/DiscordYugiohDeckBot /app/DiscordYugiohDeckBot

CMD ["/app/DiscordYugiohDeckBot"]
