use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize)]
struct ResponseHeader {
    status: u64,
    #[serde(rename = "QTime")]
    qtime: u64
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Body {
    #[serde(rename = "responseHeader")]
    response_header: ResponseHeader
}