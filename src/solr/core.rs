use anyhow::Result;

pub async fn create(name: String) -> Result<()> {
    reqwest::Client::new().post(format!("http://localhost:8983/solr/admin/cores?action=CREATE&name={}&configSet={}/solr/configsets/squid", name, super::properties().await?.system_properties.jetty_home))
        .send()
        .await?;

    Ok(())
}
