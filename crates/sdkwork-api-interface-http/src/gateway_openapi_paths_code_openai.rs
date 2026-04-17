use super::*;

#[utoipa::path(
        get,
        path = "/v1/models",
        tag = "code.openai",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible model catalog.", body = ListModelsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load model catalog.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn list_models() {}

#[utoipa::path(
        get,
        path = "/v1/models/{model_id}",
        tag = "code.openai",
        params(("model_id" = String, Path, description = "Model identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible model metadata.", body = sdkwork_api_contract_openai::models::ModelObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested model was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load model metadata.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn get_model() {}

#[utoipa::path(
        delete,
        path = "/v1/models/{model_id}",
        tag = "code.openai",
        params(("model_id" = String, Path, description = "Model identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted model.", body = sdkwork_api_contract_openai::models::DeleteModelResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested model was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the model.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn model_delete() {}

#[utoipa::path(
        get,
        path = "/v1/chat/completions",
        tag = "code.openai",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible chat completions.", body = sdkwork_api_contract_openai::chat_completions::ListChatCompletionsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load chat completions.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn chat_completions_list() {}

#[utoipa::path(
        post,
        path = "/v1/chat/completions",
        tag = "code.openai",
        request_body = CreateChatCompletionRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Chat completion response.", body = ChatCompletionResponse),
            (status = 400, description = "Invalid completion payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the chat completion.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn chat_completions() {}

#[utoipa::path(
        get,
        path = "/v1/chat/completions/{completion_id}",
        tag = "code.openai",
        params(("completion_id" = String, Path, description = "Chat completion identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible chat completion metadata.", body = ChatCompletionResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested chat completion was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the chat completion.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn chat_completion_get() {}

#[utoipa::path(
        post,
        path = "/v1/chat/completions/{completion_id}",
        tag = "code.openai",
        params(("completion_id" = String, Path, description = "Chat completion identifier.")),
        request_body = sdkwork_api_contract_openai::chat_completions::UpdateChatCompletionRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated chat completion.", body = ChatCompletionResponse),
            (status = 400, description = "Invalid chat completion update payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested chat completion was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to update the chat completion.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn chat_completion_update() {}

#[utoipa::path(
        delete,
        path = "/v1/chat/completions/{completion_id}",
        tag = "code.openai",
        params(("completion_id" = String, Path, description = "Chat completion identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted chat completion.", body = sdkwork_api_contract_openai::chat_completions::DeleteChatCompletionResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested chat completion was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the chat completion.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn chat_completion_delete() {}

#[utoipa::path(
        get,
        path = "/v1/chat/completions/{completion_id}/messages",
        tag = "code.openai",
        params(("completion_id" = String, Path, description = "Chat completion identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible chat completion messages.", body = sdkwork_api_contract_openai::chat_completions::ListChatCompletionMessagesResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested chat completion was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load chat completion messages.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn chat_completion_messages_list() {}

#[utoipa::path(
        post,
        path = "/v1/completions",
        tag = "code.openai",
        request_body = CreateCompletionRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Text completion response.", body = CompletionObject),
            (status = 400, description = "Invalid completion payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the completion.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn completions() {}

#[utoipa::path(
        post,
        path = "/v1/responses",
        tag = "code.openai",
        request_body = CreateResponseRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Response generation result.", body = ResponseObject),
            (status = 400, description = "Invalid response payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the response.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn responses() {}

#[utoipa::path(
        post,
        path = "/v1/responses/input_tokens",
        tag = "code.openai",
        request_body = CountResponseInputTokensRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Response input token count result.", body = ResponseInputTokensObject),
            (status = 400, description = "Invalid response token count payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to count response input tokens.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn responses_input_tokens() {}

#[utoipa::path(
        post,
        path = "/v1/responses/compact",
        tag = "code.openai",
        request_body = CompactResponseRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Response compaction result.", body = ResponseCompactionObject),
            (status = 400, description = "Invalid response compaction payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to compact the response.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn responses_compact() {}

#[utoipa::path(
        get,
        path = "/v1/responses/{response_id}",
        tag = "code.openai",
        params(("response_id" = String, Path, description = "Response identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible response.", body = ResponseObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested response was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the response.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn response_get() {}

#[utoipa::path(
        delete,
        path = "/v1/responses/{response_id}",
        tag = "code.openai",
        params(("response_id" = String, Path, description = "Response identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted response.", body = DeleteResponseResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested response was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the response.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn response_delete() {}

#[utoipa::path(
        get,
        path = "/v1/responses/{response_id}/input_items",
        tag = "code.openai",
        params(("response_id" = String, Path, description = "Response identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible response input items.", body = ListResponseInputItemsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested response was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load response input items.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn response_input_items() {}

#[utoipa::path(
        post,
        path = "/v1/responses/{response_id}/cancel",
        tag = "code.openai",
        params(("response_id" = String, Path, description = "Response identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Cancelled response.", body = ResponseObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested response was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to cancel the response.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn response_cancel() {}

#[utoipa::path(
        post,
        path = "/v1/embeddings",
        tag = "code.openai",
        request_body = CreateEmbeddingRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Embedding generation result.", body = CreateEmbeddingResponse),
            (status = 400, description = "Invalid embedding payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create embeddings.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn embeddings() {}

#[utoipa::path(
        post,
        path = "/v1/moderations",
        tag = "code.openai",
        request_body = CreateModerationRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Moderation result.", body = ModerationResponse),
            (status = 400, description = "Invalid moderation payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the moderation.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn moderations() {}
