//! fly-io-metadata capability provider
//!
//!
use anyhow::{anyhow, bail, Context as _};
use flytrap::{Instance, Resolver};
use tracing::info;
use wasmcloud_provider_sdk::core::HostData;
use wasmcloud_provider_sdk::{
    get_connection, load_host_data, run_provider, Context, LinkConfig, Provider,
};

use cosmonic_labs::cloud_metadata::types::{Error, Metadata, Region};

wit_bindgen_wrpc::generate!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    FlyIoMetadataProvider::run().await?;
    eprintln!("Long running operation provider exiting");
    Ok(())
}

#[derive(Default, Clone)]
struct FlyIoMetadataProvider {}

impl FlyIoMetadataProvider {
    async fn run() -> anyhow::Result<()> {
        let host_data = load_host_data().context("failed to load host data")?;
        let provider = Self::from_host_data(host_data);
        let shutdown = run_provider(provider.clone(), "operations-provider")
            .await
            .context("failed to run provider")?;
        let connection = get_connection();
        eprintln!("here");
        serve(
            &connection.get_wrpc_client(connection.provider_key()),
            provider,
            shutdown,
        )
        .await
    }

    pub fn from_host_data(_host_data: &HostData) -> FlyIoMetadataProvider {
        FlyIoMetadataProvider::default()
    }
}

impl Provider for FlyIoMetadataProvider {
    async fn receive_link_config_as_target(
        &self,
        LinkConfig {
            source_id, config, ..
        }: LinkConfig<'_>,
    ) -> anyhow::Result<()> {
        info!(
            source_id,
            "received link configuration for component {:?}", config
        );
        Ok(())
    }

    async fn delete_link(&self, _source_id: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

impl exports::cosmonic_labs::cloud_metadata::service::Handler<Option<Context>>
    for FlyIoMetadataProvider
{
    async fn get(&self, _ctx: Option<Context>) -> anyhow::Result<Result<Metadata, Error>> {
        let hostname = hostname::get()?.to_str().unwrap().to_string();
        let resolver = Resolver::new().map_err(|e| Error {
            message: format!("failed to build resolver: {e}"),
        })?;
        // TODO can optimize this by only looking at a particular app using a linkdef.
        let instances = resolver.instances().await.map_err(|e| Error {
            message: format!("failed to resolve instances: {e}"),
        })?;
        let instance: Vec<&Instance> = instances
            .iter()
            .filter(|instance| *instance.node.id == hostname)
            .collect();
        if instance.len() != 1 {
            bail!("somehow ended up finding more than one or 0 matches".to_string());
        }

        let instance = instance[0];
        let private_ip = instance.private_ip.clone().to_string();
        let region = match instance.node.region() {
            Some(r) => r,
            None => bail!("failed to get region"),
        };

        let region_rec = Region {
            city: Some(format!("{} ({})", region.city.name, region.city.country)),
            code: Some(region.code.to_string()),
            name: region.name.to_string(),
        };

        let metadata = Metadata {
            region: region_rec,
            public_ip: None,
            private_ip,
            id: hostname.clone(),
        };

        Ok(Ok(metadata))
    }
}
