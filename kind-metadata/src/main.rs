//! kind-metadata capability provider
//!
//!
use anyhow::{anyhow, bail, Context as _};
use tracing::info;
use wasmcloud_provider_sdk::core::HostData;
use wasmcloud_provider_sdk::{
    get_connection, load_host_data, run_provider, Context, LinkConfig, Provider,
};

use cosmonic_labs::cloud_metadata::types::{Error, Metadata, Region};

wit_bindgen_wrpc::generate!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    KindMetadataProvider::run().await?;
    eprintln!("Long running operation provider exiting");
    Ok(())
}

const REGION_NAME: &str = "kind";

#[derive(Default, Clone)]
struct KindMetadataProvider {}

impl KindMetadataProvider {
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

    pub fn from_host_data(_host_data: &HostData) -> KindMetadataProvider {
        KindMetadataProvider::default()
    }
}

impl Provider for KindMetadataProvider {
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
    for KindMetadataProvider
{
    async fn get(&self, _ctx: Option<Context>) -> anyhow::Result<Result<Metadata, Error>> {
        let hostname = hostname::get()?.to_str().unwrap().to_string();

        let region_rec = Region {
            name: REGION_NAME.to_string(),
            code: Some(REGION_NAME.to_string()),
            city: None,
        };

        let ip = std::env::var("HOST_IP").unwrap_or_default();

        let metadata = Metadata {
            region: region_rec,
            public_ip: None,
            private_ip: ip,
            id: hostname.clone(),
        };

        Ok(Ok(metadata))
    }
}
