FROM rust AS builder
WORKDIR /usr/src
RUN rustup target add x86_64-unknown-linux-musl

ENV TARGET_CC=gcc
WORKDIR /usr/src/app
COPY c8y-ms-sdk/src ./c8y-ms-sdk/src
COPY c8y-ms-sdk/Cargo.toml ./c8y-ms-sdk/Cargo.toml
COPY c8y-ms-sdk/Cargo.lock ./c8y-ms-sdk/Cargo.lock
COPY c8y-sdk/src ./c8y-sdk/src
COPY c8y-sdk/Cargo.toml ./c8y-sdk/Cargo.toml
COPY c8y-sdk/Cargo.lock ./c8y-sdk/Cargo.lock
COPY rust-ms/src ./rust-ms/src
COPY rust-ms/Cargo.toml ./rust-ms/Cargo.toml
COPY rust-ms/Cargo.lock ./rust-ms/Cargo.lock
WORKDIR /usr/src/app/rust-ms
RUN cargo build --target x86_64-unknown-linux-musl --release 

FROM scratch
COPY --from=builder /usr/src/app/rust-ms/target/x86_64-unknown-linux-musl/release/rust-ms .
CMD ["./rust-ms"]
