use reqwest::header;

use crate::{server::ServerClient, zones::ZoneClient};

pub struct Client {
    pub(crate) base_url: String,
    pub(crate) server_name: String,
    pub(crate) http_client: reqwest::Client,
}

impl Client {
    pub fn new(base_url: &str, server_name: &str, api_token: &str) -> Self {
        let mut headers = header::HeaderMap::new();
        let mut auth_header = header::HeaderValue::from_str(api_token).unwrap();
        auth_header.set_sensitive(true);
        headers.insert("X-API-Key", auth_header);
        let accept_header = header::HeaderValue::from_static("application/json");
        headers.insert(header::ACCEPT, accept_header);
    
        let http_client = reqwest::Client::builder()
            .user_agent("powerdns.rs/0.1")
            .default_headers(headers)
            .build()
            .unwrap();

        Client {
            base_url: base_url.to_string(),
            server_name: server_name.to_string(),
            http_client,
        }
    }

    pub fn server(&self) -> ServerClient {
        ServerClient::new(self)
    }

    pub fn zone(&self) -> ZoneClient {
        ZoneClient::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::client::Client;
    use std::env;

    #[test]
    fn build_client() {
        let _client = Client::new(
            &env::var("PDNS_HOST").unwrap_or_else(|_| String::from("http://localhost:8081")),
            &env::var("PDNS_SERVER").unwrap_or_else(|_| String::from("localhost")),
            &env::var("PDNS_API_KEY").unwrap(),
        );
    }
}
