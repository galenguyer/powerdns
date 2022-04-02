use serde::Deserialize;

use crate::Client;
use crate::Error;

/// A Zone object represents an authoritative DNS Zone.
#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde_with::skip_serializing_none]
pub struct Zone {
    /// Opaque zone id (string), assigned by the server, should not be
    /// interpreted by the application. Guaranteed to be safe for embedding in
    /// URLs.
    pub id: String,
    /// Name of the zone (e.g. “example.com.”) MUST have a trailing dot
    pub name: String,
    /// Set to “Zone”
    #[serde(rename = "type")]
    pub type_field: String,
    /// API endpoint for this zone
    pub url: String,
    /// Zone kind, one of “Native”, “Master”, “Slave”
    pub kind: String,
    /// RRSets in this zone (for zones/{zone_id} endpoint only; omitted during
    /// GET on the …/zones list endpoint)
    pub rrsets: Vec<RRSet>,
    /// The SOA serial number
    pub serial: u32,
    /// The SOA serial notifications have been sent out for
    pub notified_serial: u32,
    /// The SOA serial as seen in query responses. Calculated using the SOA-EDIT
    /// metadata, default-soa-edit and default-soa-edit-signed settings
    pub edited_serial: u32,
    /// List of IP addresses configured as a master for this zone (“Slave” type
    /// zones only)
    pub masters: Vec<String>,
    /// Whether or not this zone is DNSSEC signed (inferred from presigned being
    /// true XOR presence of at least one cryptokey with active being true)
    pub dnssec: bool,
    /// The NSEC3PARAM record
    pub nsec3param: String,
    /// Whether or not the zone uses NSEC3 narrow
    pub nsec3narrow: bool,
    /// Whether or not the zone is pre-signed
    pub presigned: bool,
    /// The SOA-EDIT metadata item
    pub soa_edit: String,
    /// The SOA-EDIT-API metadata item
    pub soa_edit_api: String,
    /// Whether or not the zone will be rectified on data changes via the API
    pub api_rectify: bool,
    /// MAY contain a BIND-style zone file when creating a zone
    pub zone: String,
    /// MAY be set. Its value is defined by local policy
    pub account: String,
    /// MAY be sent in client bodies during creation, and MUST NOT be sent by
    /// the server. Simple list of strings of nameserver names, including the
    /// trailing dot. Not required for slave zones.
    pub nameservers: Vec<String>,
    /// The id of the TSIG keys used for master operation in this zone
    pub master_tsig_key_ids: Vec<String>,
    /// The id of the TSIG keys used for slave operation in this zone
    pub slave_tsig_key_ids: Vec<String>,
}

/// This represents a Resource Record Set (all records with the same name and
/// type).
#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
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
    pub changetype: String,
    /// All records in this RRSet. When updating Records, this is the list of
    /// new records (replacing the old ones). Must be empty when changetype is
    /// set to DELETE. An empty list results in deletion of all records (and
    /// comments).
    pub records: Vec<Record>,
    /// List of Comment. Must be empty when changetype is set to DELETE. An
    /// empty list results in deletion of all comments. modified_at is optional
    /// and defaults to the current server time.
    pub comments: Vec<Comment>,
}

/// The RREntry object represents a single record.
#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde_with::skip_serializing_none]
pub struct Record {
    /// The content of this record
    pub content: String,
    /// Whether or not this record is disabled. When unset, the record is not
    /// disabled
    pub disabled: bool,
}

/// A comment about an RRSet.
#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
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
            Ok(resp.json::<Vec<Zone>>().await.unwrap())
        } else {
            Err(resp.json::<Error>().await.unwrap())
        }
    }


    pub async fn get(&self, name: &str) -> Zone {
        let resp = self
            .api_client
            .http_client
            .get(format!(
                "{}/api/v1/servers/{}/zones/{name}",
                self.api_client.base_url, self.api_client.server_name
            ))
            .send()
            .await
            .unwrap();

        resp.json::<Zone>().await.unwrap()
    }
}
