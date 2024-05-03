# fly-io-metadata Capability Provider

This capability provider implements the
[cloud-metadata](../cloud-metadata/README.md) capability for the Fly.io
platform. It provides metadata about the Fly.io environment in which the
wasmCloud host is running on by querying the [Fly.io inetrnal
address](https://fly.io/docs/networking/private-networking/#fly-io-internal-addresses)
associated with the app.

