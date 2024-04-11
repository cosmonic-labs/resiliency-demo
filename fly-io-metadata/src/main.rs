//! fly-io-metadata capability provider
//!
//!
use fly_metadata::*;
use flytrap::{Instance, Resolver};
use wasmbus_rpc::provider::prelude::*;

// main (via provider_main) initializes the threaded tokio executor,
// listens to lattice rpcs, handles actor links,
// and returns only when it receives a shutdown message
//
fn main() -> Result<(), Box<dyn std::error::Error>> {
    provider_main(
        FlyIoMetadataProvider::default(),
        Some("FlyIoMetadata".to_string()),
    )?;

    eprintln!("fly-io-metadata provider exiting");
    Ok(())
}

/// fly-io-metadata capability provider implementation
#[derive(Default, Clone, Provider)]
#[services(Metadata)]
struct FlyIoMetadataProvider {}

/// use default implementations of provider message handlers
impl ProviderDispatch for FlyIoMetadataProvider {}
impl ProviderHandler for FlyIoMetadataProvider {}

#[async_trait]
impl Metadata for FlyIoMetadataProvider {
    async fn get(&self, _ctx: &Context) -> RpcResult<GetResponse> {
        let hostname = hostname::get()?.to_str().unwrap().to_string();
        let resolver = Resolver::new()
            .map_err(|e| RpcError::Other(format!("failed to build resolver: {e}")))?;
        // TODO can optimize this by only looking at a particular app using a linkdef.
        let instances = resolver
            .instances()
            .await
            .map_err(|e| RpcError::Other(format!("failed to resolve instances: {e}")))?;
        let instance: Vec<&Instance> = instances
            .iter()
            .filter(|instance| *instance.node.id == hostname)
            .collect();
        if instance.len() != 1 {
            return Err(RpcError::Other(
                "somehow ended up finding more than one or 0 matches".to_string(),
            ));
        }

        let instance = instance[0];
        let app = instance.app.clone();
        let private_ip = instance.private_ip.clone().to_string();
        let region = match instance.node.region() {
            Some(r) => r,
            None => return Err(RpcError::Other("failed to get region".to_string())),
        };

        let region_info = Region {
            city: format!("{} ({})", region.city.name, region.city.country),
            code: region.code.to_string(),
            name: region.name.to_string(),
        };

        let metadata = GetResponse {
            app_name: app,
            region: region_info,
            private_ip,
            machine_id: hostname.clone(),
        };
        Ok(metadata)
    }
}
