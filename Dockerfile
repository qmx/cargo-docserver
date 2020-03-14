FROM qmxme/curl as builder
ARG TARGETARCH
ARG TARGETVARIANT
ARG REF
RUN curl -o /usr/local/bin/cargo-docserver -L https://github.com/qmx/cargo-docserver/releases/download/$REF/cargo-docserver-linux-$TARGETARCH$TARGETVARIANT

FROM alpine:3.11
COPY --from=builder /usr/local/bin/* /opt/rust-tools/bin/
