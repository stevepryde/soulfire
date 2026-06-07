use std::fmt::Display;

use base64::prelude::*;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Url {
    url: String,
    query: Vec<String>,
}

#[allow(dead_code)]
impl Url {
    pub fn new(url: impl ToString) -> Self {
        Self {
            url: url.to_string(),
            query: Vec::new(),
        }
    }

    pub fn with_query(mut self, key: impl Display, value: impl Display) -> Self {
        self.add_query(key, value);
        self
    }

    pub fn with_query_from(mut self, query: impl IntoQuery) -> Self {
        self.add_query_from(query);
        self
    }

    pub fn add_query(&mut self, key: impl Display, value: impl Display) {
        self.query.push(format!("{key}={value}"));
    }

    pub fn add_query_from<T: IntoQuery>(&mut self, query: T) {
        self.query
            .extend(query.into_query().iter().map(|(k, v)| format!("{k}={v}")));
    }

    pub fn build(mut self) -> String {
        if !self.query.is_empty() {
            self.url.push('?');
            self.url.push_str(&self.query.join("&"));
        }
        self.url
    }
}

pub trait IntoQuery {
    fn into_query(self) -> Vec<(String, String)>;
}

#[allow(dead_code)] // vendored helper; not yet called by the trimmed OpenAI path
pub fn base64_encode(data: &[u8]) -> String {
    BASE64_STANDARD.encode(data)
}

#[allow(dead_code)]
pub fn base64_decode(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
    BASE64_STANDARD.decode(data.as_bytes())
}
