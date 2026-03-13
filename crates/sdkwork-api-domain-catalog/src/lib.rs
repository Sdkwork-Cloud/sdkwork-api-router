use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
}

impl Channel {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxyProvider {
    pub id: String,
    pub channel_id: String,
    pub adapter_kind: String,
    pub base_url: String,
    pub display_name: String,
    #[serde(default)]
    pub channel_bindings: Vec<ProviderChannelBinding>,
}

impl ProxyProvider {
    pub fn new(
        id: impl Into<String>,
        channel_id: impl Into<String>,
        adapter_kind: impl Into<String>,
        base_url: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Self {
        let id = id.into();
        let channel_id = channel_id.into();
        Self {
            channel_bindings: vec![ProviderChannelBinding::primary(
                id.clone(),
                channel_id.clone(),
            )],
            id,
            channel_id,
            adapter_kind: adapter_kind.into(),
            base_url: base_url.into(),
            display_name: display_name.into(),
        }
    }

    pub fn with_channel_binding(mut self, binding: ProviderChannelBinding) -> Self {
        let mut binding = binding.with_provider_id(self.id.clone());
        binding.is_primary = binding.channel_id == self.channel_id;

        if let Some(existing) = self
            .channel_bindings
            .iter_mut()
            .find(|existing| existing.channel_id == binding.channel_id)
        {
            *existing = binding;
        } else {
            self.channel_bindings.push(binding);
        }

        self.channel_bindings
            .sort_by_key(|binding| (!binding.is_primary, binding.channel_id.clone()));
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderChannelBinding {
    pub provider_id: String,
    pub channel_id: String,
    #[serde(default)]
    pub is_primary: bool,
}

impl ProviderChannelBinding {
    pub fn new(provider_id: impl Into<String>, channel_id: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
            channel_id: channel_id.into(),
            is_primary: false,
        }
    }

    pub fn primary(provider_id: impl Into<String>, channel_id: impl Into<String>) -> Self {
        Self {
            is_primary: true,
            ..Self::new(provider_id, channel_id)
        }
    }

    pub fn with_provider_id(mut self, provider_id: impl Into<String>) -> Self {
        self.provider_id = provider_id.into();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelCapability {
    ChatCompletions,
    Responses,
    Embeddings,
    Completions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelCatalogEntry {
    pub external_name: String,
    pub provider_id: String,
    #[serde(default)]
    pub capabilities: Vec<ModelCapability>,
    #[serde(default)]
    pub streaming: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_window: Option<u64>,
}

impl ModelCatalogEntry {
    pub fn new(external_name: impl Into<String>, provider_id: impl Into<String>) -> Self {
        Self {
            external_name: external_name.into(),
            provider_id: provider_id.into(),
            capabilities: Vec::new(),
            streaming: false,
            context_window: None,
        }
    }

    pub fn with_capability(mut self, capability: ModelCapability) -> Self {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
        self
    }

    pub fn with_streaming(mut self, streaming: bool) -> Self {
        self.streaming = streaming;
        self
    }

    pub fn with_context_window(mut self, context_window: u64) -> Self {
        self.context_window = Some(context_window);
        self
    }
}

pub type ModelVariant = ModelCatalogEntry;
