//! gcp-metadata provider
//!

use anyhow::Context as _;
use serde::Deserialize;
use tracing::info;
use wasmcloud_provider_sdk::core::HostData;
use wasmcloud_provider_sdk::{
    get_connection, load_host_data, run_provider, Context, LinkConfig, Provider,
};

use cosmonic_labs::cloud_metadata::types::{Error, Metadata, Region};

wit_bindgen_wrpc::generate!();

const METADATA_ENDPOINT: &str =
    "http://metadata.google.internal/computeMetadata/v1/instance/?recursive=true";

#[derive(Deserialize)]
/// Partial representation of the GCP metadata endpoint
struct GCPMetadata {
    pub zone: String,
    pub name: String,
    #[serde(rename = "networkInterfaces", default)]
    pub interfaces: Vec<Interfaces>,
}

#[derive(Deserialize, Default)]
struct Interfaces {
    #[serde(rename = "accessConfigs")]
    access_configs: Vec<AccessConfig>,
    ip: String,
}

#[derive(Deserialize, Default)]
struct AccessConfig {
    #[serde(rename = "externalIp")]
    external_ip: String,
}

impl From<GCPMetadata> for Metadata {
    fn from(meta: GCPMetadata) -> Self {
        let region = meta.zone.split('/').last().unwrap().to_string();
        let public_ip = match meta.interfaces.first() {
            Some(interface) => interface
                .access_configs
                .first()
                .map(|ac| ac.external_ip.clone()),
            None => None,
        };

        let private_ip = match meta.interfaces.first() {
            Some(interface) => interface.ip.clone(),
            None => "".to_string(),
        };

        Metadata {
            id: meta.name,
            region: Region {
                name: region.clone(),
                code: Some(region),
                city: None,
            },
            public_ip,
            private_ip,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    GCPMetadataProvider::run().await?;
    eprintln!("gcp metadata provider exiting");
    Ok(())
}

#[derive(Default, Clone)]
struct GCPMetadataProvider {}
impl GCPMetadataProvider {
    async fn run() -> anyhow::Result<()> {
        let host_data = load_host_data().context("failed to load host data")?;
        let provider = Self::from_host_data(host_data);
        let shutdown = run_provider(provider.clone(), "gcp-metadata")
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

    pub fn from_host_data(_host_data: &HostData) -> GCPMetadataProvider {
        GCPMetadataProvider::default()
    }
}

impl Provider for GCPMetadataProvider {
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
    for GCPMetadataProvider
{
    async fn get(&self, _ctx: Option<Context>) -> anyhow::Result<Result<Metadata, Error>> {
        let client = reqwest::Client::new();
        let instance_meta = client
            .get(METADATA_ENDPOINT)
            .header("Metadata-Flavor", "Google")
            .send()
            .await
            .context("failed to fetch metadata")?
            .json::<GCPMetadata>()
            .await
            .context("failed to parse metadata")?;

        let metadata = Metadata::from(instance_meta);
        Ok(Ok(metadata))
    }
}
