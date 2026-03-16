use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretEnvelope {
    pub ciphertext: String,
    pub key_version: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretBackendKind {
    #[default]
    DatabaseEncrypted,
    LocalEncryptedFile,
    OsKeyring,
}

impl SecretBackendKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DatabaseEncrypted => "database_encrypted",
            Self::LocalEncryptedFile => "local_encrypted_file",
            Self::OsKeyring => "os_keyring",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "database_encrypted" => Ok(Self::DatabaseEncrypted),
            "local_encrypted_file" => Ok(Self::LocalEncryptedFile),
            "os_keyring" => Ok(Self::OsKeyring),
            other => Err(anyhow!("unsupported secret backend: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CredentialSecretRef {
    pub tenant_id: String,
    pub provider_id: String,
    pub key_reference: String,
}

impl CredentialSecretRef {
    pub fn new(
        tenant_id: impl Into<String>,
        provider_id: impl Into<String>,
        key_reference: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            provider_id: provider_id.into(),
            key_reference: key_reference.into(),
        }
    }

    pub fn storage_key(&self) -> String {
        format!(
            "{}::{}::{}",
            self.tenant_id, self.provider_id, self.key_reference
        )
    }
}

pub fn encrypt(master_key: &str, plaintext: &str) -> Result<SecretEnvelope> {
    let payload = format!("{master_key}:{plaintext}");
    Ok(SecretEnvelope {
        ciphertext: STANDARD.encode(payload),
        key_version: 1,
    })
}

pub fn decrypt(master_key: &str, envelope: &SecretEnvelope) -> Result<String> {
    let decoded = STANDARD.decode(&envelope.ciphertext)?;
    let decoded = String::from_utf8(decoded)?;
    let prefix = format!("{master_key}:");
    decoded
        .strip_prefix(&prefix)
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("invalid master key"))
}

pub fn master_key_id(master_key: &str) -> String {
    let digest = Sha256::digest(master_key.as_bytes());
    let mut encoded = String::with_capacity("sha256:".len() + digest.len() * 2);
    encoded.push_str("sha256:");
    for byte in digest {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
}
