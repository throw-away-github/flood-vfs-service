FROM rust:1.68.2 as builder
WORKDIR /app

COPY . .
RUN cargo build --release

FROM debian:buster-slim

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m appuser
USER appuser

COPY --from=builder /app/target/release/flood-vfs-service /usr/local/bin/

ENV ENDPOINT ""
ENV POLL_INTERVAL 30
ENV RCLONE_REMOTE ""
ENV MOUNT_DIRECTORY ""



#EXPOSE 8080

CMD ["flood-vfs-service"]
