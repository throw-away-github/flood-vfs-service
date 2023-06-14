# stage 1: generate a recipe file for cargo
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# stage 2: build the dependencies
FROM chef AS cacher
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 3: build the app using the cached dependencies
FROM chef as builder
# copy the source code
COPY . .
# copy the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
# build environment variables
ENV STATIC_PATH ./public
# build the app
RUN cargo build --release
RUN strip target/release/flood-vfs-service

# Stage 4: copy the binary and extra files to a distroless image
FROM debian:buster-slim
# add ca-certificates and tzdata for timezones
RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*
# create a non-root user to run the app
RUN useradd -m appuser
USER appuser
# copy the binary from the builder stage
COPY --from=builder /app/target/release/flood-vfs-service /usr/local/bin/
# set the default environment variables
ENV ENDPOINT ""
ENV POLL_INTERVAL 30
ENV RCLONE_REMOTE ""
ENV MOUNT_DIRECTORY ""
ENV LOG_LEVEL "INFO"

# set the default command to run when starting the container
CMD ["flood-vfs-service"]
