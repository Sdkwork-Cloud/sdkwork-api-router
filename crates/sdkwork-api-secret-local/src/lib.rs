use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use sdkwork_api_secret_core::{CredentialSecretRef, SecretEnvelope};
use serde::{Deserialize, Serialize};

pub fn backend_name() -> &'static str {
    "local_encrypted_file"
}

#[derive(Debug, Clone)]
pub struct LocalEncryptedFileSecretStore {
    path: PathBuf,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct SecretFileDocument {
    entries: HashMap<String, SecretEnvelope>,
}

impl LocalEncryptedFileSecretStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn store_envelope(
        &self,
        secret_ref: &CredentialSecretRef,
        envelope: &SecretEnvelope,
    ) -> Result<()> {
        let mut document = self.load_document()?;
        document
            .entries
            .insert(secret_ref.storage_key(), envelope.clone());
        self.write_document(&document)
    }

    pub fn load_envelope(
        &self,
        secret_ref: &CredentialSecretRef,
    ) -> Result<Option<SecretEnvelope>> {
        let document = self.load_document()?;
        Ok(document.entries.get(&secret_ref.storage_key()).cloned())
    }

    fn load_document(&self) -> Result<SecretFileDocument> {
        if !self.path.exists() {
            return Ok(SecretFileDocument::default());
        }

        let contents = fs::read_to_string(&self.path)?;
        if contents.trim().is_empty() {
            return Ok(SecretFileDocument::default());
        }

        Ok(serde_json::from_str(&contents)?)
    }

    fn write_document(&self, document: &SecretFileDocument) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let payload = serde_json::to_string_pretty(document)?;
        fs::write(&self.path, payload)?;
        Ok(())
    }
}
