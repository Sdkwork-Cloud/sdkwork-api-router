use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageRecord {
    pub project_id: String,
    pub model: String,
    pub provider: String,
    #[serde(default)]
    pub units: u64,
    #[serde(default)]
    pub amount: f64,
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub total_tokens: u64,
    #[serde(default)]
    pub created_at_ms: u64,
}

impl UsageRecord {
    pub fn new(
        project_id: impl Into<String>,
        model: impl Into<String>,
        provider: impl Into<String>,
    ) -> Self {
        Self {
            project_id: project_id.into(),
            model: model.into(),
            provider: provider.into(),
            units: 0,
            amount: 0.0,
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            created_at_ms: 0,
        }
    }

    pub fn with_metering(mut self, units: u64, amount: f64, created_at_ms: u64) -> Self {
        self.units = units;
        self.amount = amount;
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_token_usage(
        mut self,
        input_tokens: u64,
        output_tokens: u64,
        total_tokens: u64,
    ) -> Self {
        self.input_tokens = input_tokens;
        self.output_tokens = output_tokens;
        self.total_tokens = total_tokens;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageProjectSummary {
    pub project_id: String,
    pub request_count: u64,
}

impl UsageProjectSummary {
    pub fn new(project_id: impl Into<String>, request_count: u64) -> Self {
        Self {
            project_id: project_id.into(),
            request_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageProviderSummary {
    pub provider: String,
    pub request_count: u64,
    pub project_count: u64,
}

impl UsageProviderSummary {
    pub fn new(provider: impl Into<String>, request_count: u64, project_count: u64) -> Self {
        Self {
            provider: provider.into(),
            request_count,
            project_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageModelSummary {
    pub model: String,
    pub request_count: u64,
    pub provider_count: u64,
}

impl UsageModelSummary {
    pub fn new(model: impl Into<String>, request_count: u64, provider_count: u64) -> Self {
        Self {
            model: model.into(),
            request_count,
            provider_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageSummary {
    pub total_requests: u64,
    pub project_count: u64,
    pub model_count: u64,
    pub provider_count: u64,
    pub projects: Vec<UsageProjectSummary>,
    pub providers: Vec<UsageProviderSummary>,
    pub models: Vec<UsageModelSummary>,
}

impl UsageSummary {
    pub fn empty() -> Self {
        Self {
            total_requests: 0,
            project_count: 0,
            model_count: 0,
            provider_count: 0,
            projects: Vec::new(),
            providers: Vec::new(),
            models: Vec::new(),
        }
    }
}
