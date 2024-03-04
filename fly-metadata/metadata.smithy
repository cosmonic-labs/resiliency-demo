// metadata.smithy
// A simple service that returns instance metadata from a Fly Machine


// Tell the code generator how to reference symbols defined in this namespace
metadata package = [ { namespace: "protochron.metadata", crate: "fly_metadata" } ]

namespace protochron.metadata

use org.wasmcloud.model#wasmbus
use org.wasmcloud.model#U32

/// The Metadata service has a single method, Get, which
/// retrieves a Fly Machine's metadata
@wasmbus(
    contractId: "protochron:fly_metadata",
    actorReceive: true,
    providerReceive: true )
service Metadata {
  version: "0.1",
  operations: [ Get ]
}

structure GetResponse {
    @required
    privateIP: String,
    @required
    machineID: String,
    @required
    appName: String,
    @required
    region: Region,
}

structure Region {
    @required
    code: String,
    @required
    name: String,
    @required
    city: String,
}

/// Get a Fly Machine's metadata
operation Get {
    output: GetResponse
}
