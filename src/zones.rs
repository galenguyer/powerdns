use addr::parse_domain_name;
use reqwest::{StatusCode};
use serde::{Deserialize, Serialize};

use crate::Client;
use crate::Error;
use crate::error::PowerDNSResponseError;

/// A Zone object represents an authoritative DNS Zone.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde_with::skip_serializing_none]
pub struct Zone {
    /// Opaque zone id (string), assigned by the server, should not be
    /// interpreted by the application. Guaranteed to be safe for embedding in
    /// URLs.
    pub id: Option<String>,
    /// Name of the zone (e.g. “example.com.”) MUST have a trailing dot
    pub name: Option<String>,
    /// Set to “Zone”
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    /// API endpoint for this zone
    pub url: Option<String>,
    /// Zone kind, one of “Native”, “Master”, “Slave”
    pub kind: Option<ZoneKind>,
    /// RRSets in this zone (for zones/{zone_id} endpoint only; omitted during
    /// GET on the …/zones list endpoint)
    pub rrsets: Option<Vec<RRSet>>,
    /// The SOA serial number
    pub serial: Option<u32>,
    /// The SOA serial notifications have been sent out for
    pub notified_serial: Option<u32>,
    /// The SOA serial as seen in query responses. Calculated using the SOA-EDIT
    /// metadata, default-soa-edit and default-soa-edit-signed settings
    pub edited_serial: Option<u32>,
    /// List of IP addresses configured as a master for this zone (“Slave” type
    /// zones only)
    pub masters: Option<Vec<String>>,
    /// Whether or not this zone is DNSSEC signed (inferred from presigned being
    /// true XOR presence of at least one cryptokey with active being true)
    pub dnssec: Option<bool>,
    /// The NSEC3PARAM record
    pub nsec3param: Option<String>,
    /// Whether or not the zone uses NSEC3 narrow
    pub nsec3narrow: Option<bool>,
    /// Whether or not the zone is pre-signed
    pub presigned: Option<bool>,
    /// The SOA-EDIT metadata item
    pub soa_edit: Option<String>,
    /// The SOA-EDIT-API metadata item
    pub soa_edit_api: Option<String>,
    /// Whether or not the zone will be rectified on data changes via the API
    pub api_rectify: Option<bool>,
    /// MAY contain a BIND-style zone file when creating a zone
    pub zone: Option<String>,
    /// MAY be set. Its value is defined by local policy
    pub account: Option<String>,
    /// MAY be sent in client bodies during creation, and MUST NOT be sent by
    /// the server. Simple list of strings of nameserver names, including the
    /// trailing dot. Not required for slave zones.
    pub nameservers: Option<Vec<String>>,
    /// The id of the TSIG keys used for master operation in this zone
    pub master_tsig_key_ids: Option<Vec<String>>,
    /// The id of the TSIG keys used for slave operation in this zone
    pub slave_tsig_key_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum ZoneKind {
    Native,
    Master,
    Slave,
}


/// PatchZones used to create zones with PATCH method.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PatchZone {
    pub rrsets: Vec<RRSet>
}

// impl ZoneKind {
//     fn as_str(&self) -> &'static str {
//         match self {
//             ZoneKind::Native => "Native",
//             ZoneKind::Master => "Master",
//             ZoneKind::Slave => "Slave"
//         }
//     }
// }

/// This represents a Resource Record Set (all records with the same name and type).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde_with::skip_serializing_none]
pub struct RRSet {
    /// Name for record set (e.g. “www.powerdns.com.”)
    pub name: String,
    #[serde(rename = "type")]
    /// Type of this record (e.g. “A”, “PTR”, “MX”)
    pub type_field: String,
    /// DNS TTL of the records, in seconds. MUST NOT be included when changetype
    /// is set to “DELETE”.
    pub ttl: u32,
    /// MUST be added when updating the RRSet. Must be REPLACE or DELETE. With
    /// DELETE, all existing RRs matching name and type will be deleted,
    /// including all comments. With REPLACE: when records is present, all
    /// existing RRs matching name and type will be deleted, and then new
    /// records given in records will be created. If no records are left, any
    /// existing comments will be deleted as well. When comments is present, all
    /// existing comments for the RRs matching name and type will be deleted,
    /// and then new comments given in comments will be created.
    pub changetype: Option<String>,
    /// All records in this RRSet. When updating Records, this is the list of
    /// new records (replacing the old ones). Must be empty when changetype is
    /// set to DELETE. An empty list results in deletion of all records (and
    /// comments).
    pub records: Vec<Record>,
    /// List of Comment. Must be empty when changetype is set to DELETE. An
    /// empty list results in deletion of all comments. modified_at is optional
    /// and defaults to the current server time.
    pub comments: Option<Vec<Comment>>,
}

/// The RREntry object represents a single record.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde_with::skip_serializing_none]
pub struct Record {
    /// The content of this record
    pub content: String,
    /// Whether or not this record is disabled. When unset, the record is not
    /// disabled
    pub disabled: Option<bool>,
}

/// A comment about an RRSet.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde_with::skip_serializing_none]
pub struct Comment {
    /// The actual comment
    pub content: String,
    /// Name of an account that added the comment
    pub account: String,
    /// Timestamp of the last change to the comment
    pub modified_at: u32,
}

pub struct ZoneClient<'a> {
    api_client: &'a Client,
}

impl<'a> ZoneClient<'a> {
    pub fn new(api_client: &'a Client) -> Self {
        ZoneClient { api_client }
    }

    /// List all Zones in a server
    pub async fn list(&self) -> Result<Vec<Zone>, Error> {
        let resp = self
            .api_client
            .http_client
            .get(format!(
                "{}/api/v1/servers/{}/zones",
                self.api_client.base_url, self.api_client.server_name
            ))
            .send()
            .await
            .unwrap();

        if resp.status().is_success() {
            Ok(resp.json::<Vec<Zone>>().await?)
        } else {
            Err(resp.json::<PowerDNSResponseError>().await?)?
        }
    }

    /// Get a zone managed by a server
    pub async fn get(&self, zone_id: &str) -> Result<Zone, Error> {
        let zone_id = canonicalize_domain(zone_id).unwrap();
        let resp = self
            .api_client
            .http_client
            .get(format!(
                "{}/api/v1/servers/{}/zones/{zone_id}",
                self.api_client.base_url, self.api_client.server_name
            ))
            .send()
            .await
            .unwrap();

        if resp.status().is_success() {
            Ok(resp.json::<Zone>().await?)
        } else {
            Err(resp.json::<PowerDNSResponseError>().await?)?
        }
    }

    /// Deletes this zone, all attached metadata and rrsets.
    pub async fn delete(&self, zone_id: &str) -> Result<(), Error> {
        let zone_id = canonicalize_domain(zone_id).unwrap();
        let resp = self
            .api_client
            .http_client
            .delete(format!(
                "{}/api/v1/servers/{}/zones/{zone_id}",
                self.api_client.base_url, self.api_client.server_name
            ))
            .send()
            .await
            .unwrap();

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(resp.json::<PowerDNSResponseError>().await?)?
        }
    }

    /// Patches zone, by assigning new rrsets to this zone.
    pub async fn patch(&self, zone_id: &str, zone: PatchZone) -> Result<(), Error> {
        let response = self
            .api_client
            .http_client
            .patch(
                format!("{}/api/v1/servers/{}/zones/{zone_id}",
                        self.api_client.base_url,
                        self.api_client.server_name,
                ))
            .json(&zone)
            .send()
            .await?;

        match response.status() {
            // 204 No Content – Returns 204 No Content on success.
            // 400 Bad Request – The supplied request was not valid Returns: Error object
            // 404 Not Found – Requested item was not found Returns: Error object
            // 422 Unprocessable Entity – The input to the operation was not valid Returns: Error object
            // 500 Internal Server Error – Internal server error Returns: Error object

            StatusCode::NO_CONTENT => Ok(()),
            StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND |
            StatusCode::UNPROCESSABLE_ENTITY | StatusCode::INTERNAL_SERVER_ERROR => {
                Err(Error::PowerDNS(response.json().await?))
            },
            status @ _ => Err(Error::UnexpectedStatusCode(status)),
        }
    }
}

/// Ensure a domain is canonical and top-level
fn canonicalize_domain(domain: &str) -> Result<String, ()> {
    let parsed = match parse_domain_name(domain) {
        Ok(p) => p,
        Err(_) => return Err(()),
    };

    let mut root = parsed.as_str().to_string();

    if !parsed.has_known_suffix() {
        return Err(());
    }

    if !root.ends_with('.') {
        root += ".";
    }

    Ok(root)
}

#[cfg(test)]
mod tests {
    use crate::zones::canonicalize_domain;

    #[test]
    fn already_canonical() {
        let root = canonicalize_domain("powerdns.com.").unwrap();
        assert_eq!(root, "powerdns.com.")
    }

    #[test]
    fn not_yet_canonical() {
        let root = canonicalize_domain("powerdns.com").unwrap();
        assert_eq!(root, "powerdns.com.")
    }

    #[test]
    fn not_top_level() {
        let root = canonicalize_domain("doc.powerdns.com").unwrap();
        assert_eq!(root, "doc.powerdns.com.")
    }
}
