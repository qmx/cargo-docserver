FROM qmxme/rust-builder:0.1.0 as builder
ENV CARGO_INSTALL_ROOT /opt/rust-tools
ADD . /crate
RUN cargo install --path /crate

FROM alpine:3.11
COPY --from=builder /opt/rust-tools/bin/* /opt/rust-tools/bin/
