use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFineTuningJobRequest {
    pub training_file: String,
    pub model: String,
}

impl CreateFineTuningJobRequest {
    pub fn new(training_file: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            training_file: training_file.into(),
            model: model.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FineTuningJobObject {
    pub id: String,
    pub object: &'static str,
    pub model: String,
    pub status: &'static str,
}

impl FineTuningJobObject {
    pub fn new(id: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "fine_tuning.job",
            model: model.into(),
            status: "queued",
        }
    }

    pub fn cancelled(id: impl Into<String>, model: impl Into<String>) -> Self {
        let mut job = Self::new(id, model);
        job.status = "cancelled";
        job
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ListFineTuningJobsResponse {
    pub object: &'static str,
    pub data: Vec<FineTuningJobObject>,
}

impl ListFineTuningJobsResponse {
    pub fn new(data: Vec<FineTuningJobObject>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FineTuningJobEventObject {
    pub id: String,
    pub object: &'static str,
    pub level: String,
    pub message: String,
}

impl FineTuningJobEventObject {
    pub fn new(
        id: impl Into<String>,
        level: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            object: "fine_tuning.job.event",
            level: level.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ListFineTuningJobEventsResponse {
    pub object: &'static str,
    pub data: Vec<FineTuningJobEventObject>,
}

impl ListFineTuningJobEventsResponse {
    pub fn new(data: Vec<FineTuningJobEventObject>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FineTuningJobCheckpointObject {
    pub id: String,
    pub object: &'static str,
    pub fine_tuned_model_checkpoint: String,
}

impl FineTuningJobCheckpointObject {
    pub fn new(id: impl Into<String>, fine_tuned_model_checkpoint: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "fine_tuning.job.checkpoint",
            fine_tuned_model_checkpoint: fine_tuned_model_checkpoint.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ListFineTuningJobCheckpointsResponse {
    pub object: &'static str,
    pub data: Vec<FineTuningJobCheckpointObject>,
}

impl ListFineTuningJobCheckpointsResponse {
    pub fn new(data: Vec<FineTuningJobCheckpointObject>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}
