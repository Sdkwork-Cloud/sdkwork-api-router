use super::*;

#[utoipa::path(
        post,
        path = "/v1/assistants/{assistant_id}",
        tag = "assistants",
        params(("assistant_id" = String, Path, description = "Assistant identifier.")),
        request_body = sdkwork_api_contract_openai::assistants::UpdateAssistantRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated assistant.", body = AssistantObject),
            (status = 400, description = "Invalid assistant update payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested assistant was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to update the assistant.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn assistants_update() {}

#[utoipa::path(
        delete,
        path = "/v1/assistants/{assistant_id}",
        tag = "assistants",
        params(("assistant_id" = String, Path, description = "Assistant identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted assistant.", body = sdkwork_api_contract_openai::assistants::DeleteAssistantResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested assistant was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the assistant.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn assistants_delete() {}
