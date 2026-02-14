use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Encrypted packet stored by the relay. The relay never sees the plaintext.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPacket {
    pub id: Uuid,
    pub target_instance_id: String,
    pub sender_instance_id: String,
    #[serde(with = "base64_bytes")]
    pub payload_cipher: Vec<u8>,
    #[serde(with = "base64_bytes")]
    pub nonce: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub ttl: DateTime<Utc>,
}

/// Account with rate-limit plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub api_key: String,
    pub plan: Plan,
    pub allowance: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Plan {
    Free,
    Pro,
}

/// Heartbeat registration payload.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub instance_id: String,
    pub external_ip: String,
    pub port: u16,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub ok: bool,
    pub instance_id: String,
}

/// Push packet request.
#[derive(Debug, Deserialize)]
pub struct PushRequest {
    pub target_instance_id: String,
    pub sender_instance_id: String,
    #[serde(with = "base64_bytes")]
    pub payload_cipher: Vec<u8>,
    #[serde(with = "base64_bytes")]
    pub nonce: Vec<u8>,
    pub ttl_seconds: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct PushResponse {
    pub ok: bool,
    pub packet_id: Uuid,
}

/// Pull response — list of pending packets.
#[derive(Debug, Serialize)]
pub struct PullResponse {
    pub packets: Vec<EncryptedPacket>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
}

/// Base64 serde helper for Vec<u8> fields.
mod base64_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error> {
        use base64::Engine;
        s.serialize_str(&base64::engine::general_purpose::STANDARD.encode(bytes))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        use base64::Engine;
        let s = String::deserialize(d)?;
        base64::engine::general_purpose::STANDARD
            .decode(&s)
            .map_err(serde::de::Error::custom)
    }
}
