//! digitalocean-metadata provider
//!

use anyhow::Context as _;
use tracing::info;
use wasmcloud_provider_sdk::core::HostData;
use wasmcloud_provider_sdk::{
    get_connection, load_host_data, run_provider, Context, LinkConfig, Provider,
};

use cosmonic_labs::cloud_metadata::types::{Error, Metadata, Region};

wit_bindgen_wrpc::generate!();

const METADATA_ENDPOINT: &str = "http://169.254.169.254/metadata/v1.json";

#[derive(serde::Deserialize)]
/// Partial representation of the DigitalOcean metadata endpoint
struct DOMetadata {
    pub region: String,
    #[serde(rename = "droplet_id")]
    pub id: String,
    pub interfaces: Interfaces,
}

#[derive(serde::Deserialize)]
struct Interfaces {
    pub public: Vec<Interface>,
    pub private: Vec<Interface>,
}

#[derive(serde::Deserialize)]
struct Interface {
    pub ipv4: IP,
    pub ipv6: IP,
    #[serde(rename = "type")]
    pub addr_type: String,
    pub mac: String,
}

#[derive(serde::Deserialize)]
struct IP {
    #[serde(rename = "ip_address")]
    pub address: String,
    pub netmask: String,
    pub gateway: String,
    pub cidr: u8,
}

impl From<DOMetadata> for Metadata {
    fn from(meta: DOMetadata) -> Self {
        Metadata {
            id: meta.id,
            region: Region {
                name: meta.region.clone(),
                code: Some(meta.region),
                city: None,
            },
            public_ip: Some(meta.interfaces.public[0].ipv4.address.clone()),
            private_ip: meta.interfaces.private[0].ipv4.address.clone(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    DigitalOceanMetadataProvider::run().await?;
    eprintln!("digitalocean metadata provider exiting");
    Ok(())
}

#[derive(Default, Clone)]
struct DigitalOceanMetadataProvider {}
impl DigitalOceanMetadataProvider {
    async fn run() -> anyhow::Result<()> {
        let host_data = load_host_data().context("failed to load host data")?;
        let provider = Self::from_host_data(host_data);
        let shutdown = run_provider(provider.clone(), "digitalocean-metadata")
            .await
            .context("failed to run provider")?;
        let connection = get_connection();
        serve(
            &connection.get_wrpc_client(connection.provider_key()),
            provider,
            shutdown,
        )
        .await
    }

    pub fn from_host_data(_host_data: &HostData) -> DigitalOceanMetadataProvider {
        DigitalOceanMetadataProvider::default()
    }
}

impl Provider for DigitalOceanMetadataProvider {
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
    for DigitalOceanMetadataProvider
{
    async fn get(&self, _ctx: Option<Context>) -> anyhow::Result<Result<Metadata, Error>> {
        let instance_meta = reqwest::get(METADATA_ENDPOINT)
            .await
            .context("failed to fetch metadata")?
            .json::<DOMetadata>()
            .await
            .context("failed to parse metadata")?;

        let metadata = Metadata::from(instance_meta);
        Ok(Ok(metadata))
    }
}
