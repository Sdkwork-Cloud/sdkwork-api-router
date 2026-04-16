use super::*;

#[utoipa::path(
        get,
        path = "/v1/containers",
        tag = "containers",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible containers.", body = sdkwork_api_contract_openai::containers::ListContainersResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load containers.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn containers_list() {}

#[utoipa::path(
        post,
        path = "/v1/containers",
        tag = "containers",
        request_body = sdkwork_api_contract_openai::containers::CreateContainerRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created container.", body = sdkwork_api_contract_openai::containers::ContainerObject),
            (status = 400, description = "Invalid container payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the container.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn containers_create() {}

#[utoipa::path(
        get,
        path = "/v1/containers/{container_id}",
        tag = "containers",
        params(("container_id" = String, Path, description = "Container identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible container metadata.", body = sdkwork_api_contract_openai::containers::ContainerObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested container was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the container.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn container_get() {}

#[utoipa::path(
        delete,
        path = "/v1/containers/{container_id}",
        tag = "containers",
        params(("container_id" = String, Path, description = "Container identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted container.", body = sdkwork_api_contract_openai::containers::DeleteContainerResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested container was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the container.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn container_delete() {}

#[utoipa::path(
        get,
        path = "/v1/containers/{container_id}/files",
        tag = "containers",
        params(("container_id" = String, Path, description = "Container identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible container files.", body = sdkwork_api_contract_openai::containers::ListContainerFilesResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested container was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load container files.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn container_files_list() {}

#[utoipa::path(
        post,
        path = "/v1/containers/{container_id}/files",
        tag = "containers",
        params(("container_id" = String, Path, description = "Container identifier.")),
        request_body = sdkwork_api_contract_openai::containers::CreateContainerFileRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created container file link.", body = sdkwork_api_contract_openai::containers::ContainerFileObject),
            (status = 400, description = "Invalid container file payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested container was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the container file link.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn container_files_create() {}

#[utoipa::path(
        get,
        path = "/v1/containers/{container_id}/files/{file_id}",
        tag = "containers",
        params(
            ("container_id" = String, Path, description = "Container identifier."),
            ("file_id" = String, Path, description = "Container file identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible container file metadata.", body = sdkwork_api_contract_openai::containers::ContainerFileObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested container file was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the container file.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn container_file_get() {}

#[utoipa::path(
        delete,
        path = "/v1/containers/{container_id}/files/{file_id}",
        tag = "containers",
        params(
            ("container_id" = String, Path, description = "Container identifier."),
            ("file_id" = String, Path, description = "Container file identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted container file link.", body = sdkwork_api_contract_openai::containers::DeleteContainerFileResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested container file was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the container file link.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn container_file_delete() {}

#[utoipa::path(
        get,
        path = "/v1/containers/{container_id}/files/{file_id}/content",
        tag = "containers",
        params(
            ("container_id" = String, Path, description = "Container identifier."),
            ("file_id" = String, Path, description = "Container file identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Binary container file content stream."),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested container file was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the container file content.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn container_file_content() {}

#[utoipa::path(
        get,
        path = "/v1/files/{file_id}/content",
        tag = "files",
        params(("file_id" = String, Path, description = "File identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Binary file content stream."),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested file was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the file content.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn file_content() {}
