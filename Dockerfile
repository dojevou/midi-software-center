# MIDI Software Center - Multi-stage Dockerfile
# Builds the Rust backend and pipeline tools for server/CI use
#
# Usage:
#   docker build -t midi-software-center .
#   docker run -v /path/to/midi:/data midi-software-center import /data
#
# For full Tauri desktop app, use: make build-all (native only)

# ============================================================================
# Stage 1: Rust Builder
# ============================================================================
FROM rust:1.91-bookworm AS rust-builder

# Install build dependencies
# Includes GTK/WebKit libs required by Tauri dependencies
# libc++-dev and g++ needed for unrar_sys C++ compilation
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    cmake \
    clang \
    g++ \
    libc++-dev \
    libc++abi-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libsoup-3.0-dev \
    libjavascriptcoregtk-4.1-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy workspace configuration first (for caching)
COPY Cargo.toml Cargo.lock ./
COPY .cargo/config.toml .cargo/

# Copy SQLX offline query cache (enables build without database connection)
COPY .sqlx .sqlx/

# Copy workspace member manifests
COPY shared/rust/Cargo.toml shared/rust/
COPY pipeline/src-tauri/Cargo.toml pipeline/src-tauri/
COPY scripts/import-tool/Cargo.toml scripts/import-tool/
COPY verification/Cargo.toml verification/

# Copy manifests for excluded members (needed for workspace resolution)
COPY daw/src-tauri/Cargo.toml daw/src-tauri/
COPY app/src-tauri/Cargo.toml app/src-tauri/
COPY scripts/test-midi-files/Cargo.toml scripts/test-midi-files/

# Create dummy source files for dependency caching
# Note: DAW and app need dummy files but are excluded from build
RUN mkdir -p shared/rust/src && echo "pub fn dummy() {}" > shared/rust/src/lib.rs && \
    mkdir -p pipeline/src-tauri/src && echo "fn main() {}" > pipeline/src-tauri/src/main.rs && \
    mkdir -p scripts/import-tool/src && echo "fn main() {}" > scripts/import-tool/src/main.rs && \
    mkdir -p verification/src && echo "fn main() {}" > verification/src/main.rs && \
    mkdir -p daw/src-tauri/src && echo "fn main() {}" > daw/src-tauri/src/main.rs && \
    mkdir -p daw/src-tauri/benches && echo "fn main() {}" > daw/src-tauri/benches/midi_latency.rs && \
    mkdir -p app/src-tauri/src && echo "fn main() {}" > app/src-tauri/src/main.rs && \
    mkdir -p scripts/test-midi-files/src && echo "fn main() {}" > scripts/test-midi-files/src/main.rs

# Enable SQLX offline mode (compile without database connection)
ENV SQLX_OFFLINE=true

# Build dependencies only (cached layer)
RUN cargo build --release --workspace \
    --exclude midi-daw \
    --exclude midi-software-center \
    2>/dev/null || true

# Now copy actual source code
# Remove dummy lib.rs and copy full source with proper directory structure
RUN rm -f shared/rust/src/lib.rs
COPY shared/rust/src shared/rust/src/
COPY pipeline/src-tauri/src pipeline/src-tauri/src/
COPY pipeline/src-tauri/build.rs pipeline/src-tauri/
COPY pipeline/src-tauri/tauri.conf.json pipeline/src-tauri/
COPY pipeline/src-tauri/icons pipeline/src-tauri/icons/
COPY scripts/import-tool/src scripts/import-tool/src/
COPY verification/src verification/src/

# Touch source files to invalidate cargo cache and force recompilation
RUN touch shared/rust/src/lib.rs

# Build the actual binaries
RUN cargo build --release -p midi-pipeline -p import-tool -p verification

# ============================================================================
# Stage 2: Runtime Image
# ============================================================================
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
# Note: GTK/WebKit libs only needed if running midi-pipeline GUI via xvfb
# For CLI tools (orchestrator, import_unified, etc.) only libssl3 and libpq5 are needed
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user and directories with proper permissions
RUN useradd -m -s /bin/bash midi && \
    mkdir -p /data /var/log/midi && \
    chown -R midi:midi /data /var/log/midi

# Copy binaries from builder
COPY --from=rust-builder /app/target/release/midi-pipeline /usr/local/bin/
COPY --from=rust-builder /app/target/release/import-tool /usr/local/bin/
COPY --from=rust-builder /app/target/release/midi-verification /usr/local/bin/

# Copy additional pipeline binaries
COPY --from=rust-builder /app/target/release/orchestrator /usr/local/bin/
COPY --from=rust-builder /app/target/release/import_unified /usr/local/bin/
COPY --from=rust-builder /app/target/release/parallel_extract /usr/local/bin/
COPY --from=rust-builder /app/target/release/normalize_filenames /usr/local/bin/

# Set up working directory
WORKDIR /data

# Switch to non-root user
USER midi

# Default environment variables
ENV DATABASE_URL="postgresql://midiuser:145278963@host.docker.internal:5433/midi_library"
ENV RUST_LOG="info"
ENV RUST_BACKTRACE="1"
ENV LOG_DIR="/var/log/midi"

# Healthcheck - verify CLI binaries work
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD midi-verification --help || exit 1

# Default command shows orchestrator help
# Usage: docker run midi-software-center:runtime <binary> [args]
# Examples:
#   docker run midi-software-center:runtime orchestrator --source /data
#   docker run midi-software-center:runtime import_unified /data
#   docker run midi-software-center:runtime parallel_extract --help
ENTRYPOINT ["/bin/sh", "-c"]
CMD ["orchestrator --help"]

# ============================================================================
# Stage 3: Development Image (optional)
# ============================================================================
FROM rust-builder AS development

# Install additional dev tools
RUN apt-get update && apt-get install -y \
    git \
    curl \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Install cargo tools
RUN cargo install cargo-watch cargo-tarpaulin sqlx-cli

WORKDIR /app

# Mount points for source code and data
VOLUME ["/app", "/data"]

CMD ["cargo", "watch", "-x", "check"]
