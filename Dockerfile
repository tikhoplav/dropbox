FROM rust:alpine as builder
RUN apk add --no-cache build-base
COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock
COPY src /app/src
RUN cd /app && cargo build --release

FROM alpine
COPY --from=builder /app/target/release/dropbox ./
HEALTHCHECK --interval=60s --retries=5 CMD wget --no-verbose --tries=1 --spider http://localhost/ || exit 1
CMD ["./dropbox"]