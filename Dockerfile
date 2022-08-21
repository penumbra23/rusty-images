FROM ekidd/rust-musl-builder:latest AS builder

ADD --chown=rust:rust . ./

RUN cargo build --release

FROM alpine:latest
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/rusty-images \
    /usr/local/bin/
    
ENTRYPOINT [ "/usr/local/bin/rusty-images" ]