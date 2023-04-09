pub mod core;

use serde::Deserialize;
use anyhow::Result;

#[derive(Deserialize, Debug)]
pub struct SystemProperties {
    #[serde(rename = "solr.solr.home")]
    _solr_solr_home: String,
    #[serde(rename = "solr.default.confdir")]
    _solr_default_confdir: String,
    #[serde(rename = "jetty.home")]
    pub jetty_home: String
}

#[derive(Deserialize, Debug)]
pub struct Properties {
    #[serde(rename = "system.properties")]
    pub system_properties: SystemProperties
}

pub async fn properties() -> Result<Properties> {
    Ok(
        reqwest::get("http://localhost:8983/solr/admin/info/properties?wt=json")
            .await?
            .json::<Properties>()
            .await?
    )
}