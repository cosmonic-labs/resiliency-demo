# syntax=docker/dockerfile:1
FROM debian:bookworm
ARG NATS_SERVER=2.10.14
ARG WASMCLOUD_VERSION=1.0.0
ARG TARGETARCH

RUN apt-get update && apt-get install -y runit curl && \
		curl -Lo nats-server.tar.gz "https://github.com/nats-io/nats-server/releases/download/v${NATS_SERVER}/nats-server-v${NATS_SERVER}-linux-${TARGETARCH}.tar.gz" && \
		tar -xf nats-server.tar.gz && \
		rm nats-server.tar.gz && \
		mv "nats-server-v${NATS_SERVER}-linux-${TARGETARCH}/nats-server" /usr/local/bin && \
		rm -rf "nats-server-v${NATS_SERVER}-linux-${TARGETARCH}" && \
		curl -Lo wasmcloud https://github.com/wasmCloud/wasmCloud/releases/download/v${WASMCLOUD_VERSION}/wasmcloud-x86_64-unknown-linux-musl && \
		chmod +x wasmcloud && mv wasmcloud /usr/local/bin && \
		rm -rf wasmcloud*

COPY nats /etc/service/nats
COPY wasmcloud /etc/service/wasmcloud

CMD ["/usr/bin/runsvdir", "/etc/service"]
