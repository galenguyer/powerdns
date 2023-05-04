use serde::Deserialize;

use crate::{Client, Error};
use crate::error::PowerDNSResponseError;

/// The server endpoint is the ‘basis’ for all other API operations. In the
/// PowerDNS Authoritative Server, the server_id is always localhost. However,
/// the API is written in a way that a proxy could be in front of many servers,
/// each with their own server_id.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde_with::skip_serializing_none]
pub struct Server {
    /// Set to “Server”
    #[serde(rename = "type")]
    pub type_field: String,
    /// The id of the server, “localhost”
    pub id: String,
    /// “recursor” for the PowerDNS Recursor and “authoritative” for the
    /// Authoritative Server
    pub daemon_type: String,
    /// The version of the server software
    pub version: String,
    /// The API endpoint for this server
    pub url: String,
    /// The API endpoint for this server’s configuration
    pub config_url: String,
    /// The API endpoint for this server’s zones
    pub zones_url: String,
}

pub struct ServerClient<'a> {
    api_client: &'a Client,
}

impl<'a> ServerClient<'a> {
    pub fn new(api_client: &'a Client) -> Self {
        ServerClient { api_client }
    }

    /// List all servers
    ///
    /// 200 OK – An array of servers Returns: array of Server objects
    ///
    /// 400 Bad Request – The supplied request was not valid Returns: Error
    /// object
    ///
    /// 404 Not Found – Requested item was not found Returns: Error object
    ///
    /// 422 Unprocessable Entity – The input to the operation was not valid
    /// Returns: Error object
    ///
    /// 500 Internal Server Error – Internal server error Returns: Error object
    pub async fn list(&self) -> Result<Vec<Server>, Error> {
        let resp = self
            .api_client
            .http_client
            .get(format!("{}/api/v1/servers", self.api_client.base_url))
            .send()
            .await
            .unwrap();
        if resp.status().is_success() {
            Ok(resp.json::<Vec<Server>>().await.unwrap())
        } else {
            Err(resp.json::<PowerDNSResponseError>().await?)?
        }
    }

    /// List a server
    ///
    /// # Arguments
    ///
    /// * `server_id` - The id of the server to retrieve
    ///
    /// 200 OK – An server Returns: Server object
    ///
    /// 400 Bad Request – The supplied request was not valid Returns: Error
    /// object
    ///
    /// 404 Not Found – Requested item was not found Returns: Error object
    ///
    /// 422 Unprocessable Entity – The input to the operation was not valid
    /// Returns: Error object
    ///
    /// 500 Internal Server Error – Internal server error Returns: Error object
    pub async fn get(&self, server_id: &str) -> Result<Server, Error> {
        let resp = self
            .api_client
            .http_client
            .get(format!(
                "{}/api/v1/servers/{server_id}",
                self.api_client.base_url
            ))
            .send()
            .await
            .unwrap();
        if resp.status().is_success() {
            Ok(resp.json::<Server>().await.unwrap())
        } else {
            Err(resp.json::<PowerDNSResponseError>().await?)?
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::client::Client;
    use dotenvy::dotenv;
    use std::env;

    #[tokio::test]
    async fn list() {
        dotenv().ok();
        let client = Client::new(
            &env::var("PDNS_HOST").unwrap_or_else(|_| String::from("http://localhost:8081")),
            &env::var("PDNS_SERVER").unwrap_or_else(|_| String::from("localhost")),
            &env::var("PDNS_API_KEY").unwrap(),
        );

        let server_client = client.server();

        let servers = server_client.list().await;

        assert_eq!(servers.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn get_localhost() {
        dotenv().ok();
        let client = Client::new(
            &env::var("PDNS_HOST").unwrap_or_else(|_| String::from("http://localhost:8081")),
            &env::var("PDNS_SERVER").unwrap_or_else(|_| String::from("localhost")),
            &env::var("PDNS_API_KEY").unwrap(),
        );

        let server_client = client.server();

        let server = server_client.get("localhost").await;

        assert_eq!(server.unwrap().id, "localhost");
    }
}
