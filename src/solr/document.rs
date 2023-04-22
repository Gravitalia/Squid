use anyhow::Result;

pub async fn index(id: String, text: String) -> Result<crate::models::document::create::Body> {

    // "ttl": "NOW+1DAY",
    // "expire_at": "{}"
    let response = reqwest::Client::new()
        .post("http://localhost:8983/solr/gravitalia/update?commit=true")
        .header("Content-Type", "application/json")
        .body(format!(
            r#"{{
                "add": {{
                    "doc": {{
                        "lang": "{}",
                        "sentence": "{}",
                        "id": "{}",
                    }}
                }}
            }}"#,
            crate::helpers::detect_language(text.clone())?.unwrap_or_default().to_string(),
            text,
            id,
            //(Utc::now() + Duration::seconds(5)).to_rfc3339()
        ))
        .send()
        .await?;

    Ok(response.json::<crate::models::document::create::Body>().await?)
}
