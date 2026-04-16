use super::*;

#[utoipa::path(
        get,
        path = "/v1/fine_tuning/jobs",
        tag = "fine-tuning",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible fine-tuning jobs.", body = sdkwork_api_contract_openai::fine_tuning::ListFineTuningJobsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load fine-tuning jobs.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn fine_tuning_jobs_list() {}

#[utoipa::path(
        post,
        path = "/v1/fine_tuning/jobs",
        tag = "fine-tuning",
        request_body = sdkwork_api_contract_openai::fine_tuning::CreateFineTuningJobRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created fine-tuning job.", body = sdkwork_api_contract_openai::fine_tuning::FineTuningJobObject),
            (status = 400, description = "Invalid fine-tuning job payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the fine-tuning job.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn fine_tuning_jobs_create() {}

#[utoipa::path(
        get,
        path = "/v1/fine_tuning/jobs/{fine_tuning_job_id}",
        tag = "fine-tuning",
        params(("fine_tuning_job_id" = String, Path, description = "Fine-tuning job identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible fine-tuning job metadata.", body = sdkwork_api_contract_openai::fine_tuning::FineTuningJobObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested fine-tuning job was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the fine-tuning job.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn fine_tuning_job_get() {}

#[utoipa::path(
        post,
        path = "/v1/fine_tuning/jobs/{fine_tuning_job_id}/cancel",
        tag = "fine-tuning",
        params(("fine_tuning_job_id" = String, Path, description = "Fine-tuning job identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Cancelled fine-tuning job.", body = sdkwork_api_contract_openai::fine_tuning::FineTuningJobObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested fine-tuning job was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to cancel the fine-tuning job.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn fine_tuning_job_cancel() {}

#[utoipa::path(
        get,
        path = "/v1/fine_tuning/jobs/{fine_tuning_job_id}/events",
        tag = "fine-tuning",
        params(("fine_tuning_job_id" = String, Path, description = "Fine-tuning job identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible fine-tuning job events.", body = sdkwork_api_contract_openai::fine_tuning::ListFineTuningJobEventsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested fine-tuning job was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load fine-tuning job events.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn fine_tuning_job_events() {}

#[utoipa::path(
        get,
        path = "/v1/fine_tuning/jobs/{fine_tuning_job_id}/checkpoints",
        tag = "fine-tuning",
        params(("fine_tuning_job_id" = String, Path, description = "Fine-tuning job identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible fine-tuning job checkpoints.", body = sdkwork_api_contract_openai::fine_tuning::ListFineTuningJobCheckpointsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested fine-tuning job was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load fine-tuning job checkpoints.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn fine_tuning_job_checkpoints() {}

#[utoipa::path(
        post,
        path = "/v1/fine_tuning/jobs/{fine_tuning_job_id}/pause",
        tag = "fine-tuning",
        params(("fine_tuning_job_id" = String, Path, description = "Fine-tuning job identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Paused fine-tuning job.", body = sdkwork_api_contract_openai::fine_tuning::FineTuningJobObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested fine-tuning job was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to pause the fine-tuning job.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn fine_tuning_job_pause() {}

#[utoipa::path(
        post,
        path = "/v1/fine_tuning/jobs/{fine_tuning_job_id}/resume",
        tag = "fine-tuning",
        params(("fine_tuning_job_id" = String, Path, description = "Fine-tuning job identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Resumed fine-tuning job.", body = sdkwork_api_contract_openai::fine_tuning::FineTuningJobObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested fine-tuning job was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to resume the fine-tuning job.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn fine_tuning_job_resume() {}

#[utoipa::path(
        get,
        path = "/v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions",
        tag = "fine-tuning",
        params(("fine_tuned_model_checkpoint" = String, Path, description = "Fine-tuned model checkpoint identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible fine-tuning checkpoint permissions.", body = sdkwork_api_contract_openai::fine_tuning::ListFineTuningCheckpointPermissionsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested fine-tuning checkpoint was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load fine-tuning checkpoint permissions.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn fine_tuning_checkpoint_permissions_list() {}

#[utoipa::path(
        post,
        path = "/v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions",
        tag = "fine-tuning",
        params(("fine_tuned_model_checkpoint" = String, Path, description = "Fine-tuned model checkpoint identifier.")),
        request_body = sdkwork_api_contract_openai::fine_tuning::CreateFineTuningCheckpointPermissionsRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created fine-tuning checkpoint permission.", body = sdkwork_api_contract_openai::fine_tuning::FineTuningCheckpointPermissionObject),
            (status = 400, description = "Invalid fine-tuning checkpoint permission payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested fine-tuning checkpoint was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the fine-tuning checkpoint permission.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn fine_tuning_checkpoint_permissions_create() {}

#[utoipa::path(
        delete,
        path = "/v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions/{permission_id}",
        tag = "fine-tuning",
        params(
            ("fine_tuned_model_checkpoint" = String, Path, description = "Fine-tuned model checkpoint identifier."),
            ("permission_id" = String, Path, description = "Fine-tuning checkpoint permission identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted fine-tuning checkpoint permission.", body = sdkwork_api_contract_openai::fine_tuning::DeleteFineTuningCheckpointPermissionResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested fine-tuning checkpoint permission was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the fine-tuning checkpoint permission.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn fine_tuning_checkpoint_permission_delete() {}

#[utoipa::path(
        get,
        path = "/v1/webhooks",
        tag = "webhooks",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible webhooks.", body = sdkwork_api_contract_openai::webhooks::ListWebhooksResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load webhooks.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn webhooks_list() {}

#[utoipa::path(
        post,
        path = "/v1/webhooks",
        tag = "webhooks",
        request_body = sdkwork_api_contract_openai::webhooks::CreateWebhookRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created webhook.", body = sdkwork_api_contract_openai::webhooks::WebhookObject),
            (status = 400, description = "Invalid webhook payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the webhook.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn webhooks_create() {}

#[utoipa::path(
        get,
        path = "/v1/webhooks/{webhook_id}",
        tag = "webhooks",
        params(("webhook_id" = String, Path, description = "Webhook identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible webhook metadata.", body = sdkwork_api_contract_openai::webhooks::WebhookObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested webhook was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the webhook.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn webhook_get() {}

#[utoipa::path(
        post,
        path = "/v1/webhooks/{webhook_id}",
        tag = "webhooks",
        params(("webhook_id" = String, Path, description = "Webhook identifier.")),
        request_body = sdkwork_api_contract_openai::webhooks::UpdateWebhookRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated webhook.", body = sdkwork_api_contract_openai::webhooks::WebhookObject),
            (status = 400, description = "Invalid webhook update payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested webhook was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to update the webhook.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn webhook_update() {}

#[utoipa::path(
        delete,
        path = "/v1/webhooks/{webhook_id}",
        tag = "webhooks",
        params(("webhook_id" = String, Path, description = "Webhook identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted webhook.", body = sdkwork_api_contract_openai::webhooks::DeleteWebhookResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested webhook was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the webhook.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn webhook_delete() {}

#[utoipa::path(
        get,
        path = "/v1/evals",
        tag = "evals",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible evals.", body = sdkwork_api_contract_openai::evals::ListEvalsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load evals.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn evals_list() {}

#[utoipa::path(
        post,
        path = "/v1/evals",
        tag = "evals",
        request_body = sdkwork_api_contract_openai::evals::CreateEvalRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created eval.", body = sdkwork_api_contract_openai::evals::EvalObject),
            (status = 400, description = "Invalid eval payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the eval.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn evals_create() {}

#[utoipa::path(
        get,
        path = "/v1/evals/{eval_id}",
        tag = "evals",
        params(("eval_id" = String, Path, description = "Eval identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible eval metadata.", body = sdkwork_api_contract_openai::evals::EvalObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested eval was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the eval.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn eval_get() {}

#[utoipa::path(
        post,
        path = "/v1/evals/{eval_id}",
        tag = "evals",
        params(("eval_id" = String, Path, description = "Eval identifier.")),
        request_body = sdkwork_api_contract_openai::evals::UpdateEvalRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated eval.", body = sdkwork_api_contract_openai::evals::EvalObject),
            (status = 400, description = "Invalid eval update payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested eval was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to update the eval.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn eval_update() {}

#[utoipa::path(
        delete,
        path = "/v1/evals/{eval_id}",
        tag = "evals",
        params(("eval_id" = String, Path, description = "Eval identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted eval.", body = sdkwork_api_contract_openai::evals::DeleteEvalResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested eval was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the eval.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn eval_delete() {}

#[utoipa::path(
        get,
        path = "/v1/evals/{eval_id}/runs",
        tag = "evals",
        params(("eval_id" = String, Path, description = "Eval identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible eval runs.", body = sdkwork_api_contract_openai::evals::ListEvalRunsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested eval was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load eval runs.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn eval_runs_list() {}

#[utoipa::path(
        post,
        path = "/v1/evals/{eval_id}/runs",
        tag = "evals",
        params(("eval_id" = String, Path, description = "Eval identifier.")),
        request_body = sdkwork_api_contract_openai::evals::CreateEvalRunRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created eval run.", body = sdkwork_api_contract_openai::evals::EvalRunObject),
            (status = 400, description = "Invalid eval run payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested eval was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the eval run.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn eval_runs_create() {}

#[utoipa::path(
        get,
        path = "/v1/evals/{eval_id}/runs/{run_id}",
        tag = "evals",
        params(
            ("eval_id" = String, Path, description = "Eval identifier."),
            ("run_id" = String, Path, description = "Eval run identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible eval run metadata.", body = sdkwork_api_contract_openai::evals::EvalRunObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested eval run was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the eval run.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn eval_run_get() {}

#[utoipa::path(
        delete,
        path = "/v1/evals/{eval_id}/runs/{run_id}",
        tag = "evals",
        params(
            ("eval_id" = String, Path, description = "Eval identifier."),
            ("run_id" = String, Path, description = "Eval run identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted eval run.", body = sdkwork_api_contract_openai::evals::DeleteEvalRunResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested eval run was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the eval run.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn eval_run_delete() {}

#[utoipa::path(
        post,
        path = "/v1/evals/{eval_id}/runs/{run_id}/cancel",
        tag = "evals",
        params(
            ("eval_id" = String, Path, description = "Eval identifier."),
            ("run_id" = String, Path, description = "Eval run identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Cancelled eval run.", body = sdkwork_api_contract_openai::evals::EvalRunObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested eval run was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to cancel the eval run.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn eval_run_cancel() {}

#[utoipa::path(
        get,
        path = "/v1/evals/{eval_id}/runs/{run_id}/output_items",
        tag = "evals",
        params(
            ("eval_id" = String, Path, description = "Eval identifier."),
            ("run_id" = String, Path, description = "Eval run identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible eval run output items.", body = sdkwork_api_contract_openai::evals::ListEvalRunOutputItemsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested eval run was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load eval run output items.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn eval_run_output_items_list() {}

#[utoipa::path(
        get,
        path = "/v1/evals/{eval_id}/runs/{run_id}/output_items/{output_item_id}",
        tag = "evals",
        params(
            ("eval_id" = String, Path, description = "Eval identifier."),
            ("run_id" = String, Path, description = "Eval run identifier."),
            ("output_item_id" = String, Path, description = "Eval run output item identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible eval run output item.", body = sdkwork_api_contract_openai::evals::EvalRunOutputItemObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested eval run output item was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the eval run output item.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn eval_run_output_item_get() {}
