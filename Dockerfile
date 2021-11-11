FROM messense/rust-musl-cross:x86_64-musl AS builder

RUN rustup update beta && \
    rustup target add --toolchain beta x86_64-unknown-linux-musl

RUN apt update && apt install -y musl-tools musl-dev openssl libssl-dev pkg-config

WORKDIR /app

ADD src/main.rs /app/src/main.rs
ADD Cargo.toml /app
ADD Cargo.lock /app

RUN cargo clean
ENV OPENSSL_LIB_DIR "/usr/lib/x86_64-linux-gnu"
ENV OPENSSL_INCLUDE_DIR "/usr/include/openssl"
RUN update-ca-certificates

# Create appuser
ENV USER=worker
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /app

# Copy our build
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/feedcrosspost ./

# Use an unprivileged user.
USER worker:worker

CMD ["/app/feedcrosspost"]
