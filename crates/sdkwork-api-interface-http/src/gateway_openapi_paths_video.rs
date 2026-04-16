use super::*;

#[utoipa::path(
        get,
        path = "/v1/videos",
        tag = "videos",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible videos.", body = sdkwork_api_contract_openai::videos::VideosResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load videos.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn videos_list() {}

#[utoipa::path(
        post,
        path = "/v1/videos",
        tag = "videos",
        request_body = sdkwork_api_contract_openai::videos::CreateVideoRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created video.", body = sdkwork_api_contract_openai::videos::VideoObject),
            (status = 400, description = "Invalid video payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the video.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn videos_create() {}

#[utoipa::path(
        get,
        path = "/v1/videos/{video_id}",
        tag = "videos",
        params(("video_id" = String, Path, description = "Video identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible video metadata.", body = sdkwork_api_contract_openai::videos::VideoObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested video was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the video.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_get() {}

#[utoipa::path(
        delete,
        path = "/v1/videos/{video_id}",
        tag = "videos",
        params(("video_id" = String, Path, description = "Video identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted video.", body = sdkwork_api_contract_openai::videos::DeleteVideoResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested video was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the video.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_delete() {}

#[utoipa::path(
        get,
        path = "/v1/videos/{video_id}/content",
        tag = "videos",
        params(("video_id" = String, Path, description = "Video identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Binary video content stream."),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested video was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the video content.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_content() {}

#[utoipa::path(
        post,
        path = "/v1/videos/{video_id}/remix",
        tag = "videos",
        params(("video_id" = String, Path, description = "Video identifier.")),
        request_body = sdkwork_api_contract_openai::videos::RemixVideoRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created remixed video.", body = sdkwork_api_contract_openai::videos::VideoObject),
            (status = 400, description = "Invalid video remix payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested video was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to remix the video.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_remix() {}

#[utoipa::path(
        post,
        path = "/v1/videos/characters",
        tag = "videos",
        request_body = sdkwork_api_contract_openai::videos::CreateVideoCharacterRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created video character.", body = sdkwork_api_contract_openai::videos::VideoCharacterObject),
            (status = 400, description = "Invalid video character payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the video character.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_characters_create() {}

#[utoipa::path(
        get,
        path = "/v1/videos/characters/{character_id}",
        tag = "videos",
        params(("character_id" = String, Path, description = "Video character identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible canonical video character metadata.", body = sdkwork_api_contract_openai::videos::VideoCharacterObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested video character was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the canonical video character.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_character_canonical_get() {}

#[utoipa::path(
        post,
        path = "/v1/videos/edits",
        tag = "videos",
        request_body = sdkwork_api_contract_openai::videos::EditVideoRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Edited video result.", body = sdkwork_api_contract_openai::videos::VideoObject),
            (status = 400, description = "Invalid video edit payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to edit the video.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_edits() {}

#[utoipa::path(
        post,
        path = "/v1/videos/extensions",
        tag = "videos",
        request_body = sdkwork_api_contract_openai::videos::ExtendVideoRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Extended video result.", body = sdkwork_api_contract_openai::videos::VideoObject),
            (status = 400, description = "Invalid video extension payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to extend the video.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_extensions() {}

#[utoipa::path(
        get,
        path = "/v1/videos/{video_id}/characters",
        tag = "videos",
        params(("video_id" = String, Path, description = "Video identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible video characters.", body = sdkwork_api_contract_openai::videos::VideoCharactersResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested video was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load video characters.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_characters_list() {}

#[utoipa::path(
        get,
        path = "/v1/videos/{video_id}/characters/{character_id}",
        tag = "videos",
        params(
            ("video_id" = String, Path, description = "Video identifier."),
            ("character_id" = String, Path, description = "Video character identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible video character metadata.", body = sdkwork_api_contract_openai::videos::VideoCharacterObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested video character was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the video character.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_character_get() {}

#[utoipa::path(
        post,
        path = "/v1/videos/{video_id}/characters/{character_id}",
        tag = "videos",
        params(
            ("video_id" = String, Path, description = "Video identifier."),
            ("character_id" = String, Path, description = "Video character identifier.")
        ),
        request_body = sdkwork_api_contract_openai::videos::UpdateVideoCharacterRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated video character.", body = sdkwork_api_contract_openai::videos::VideoCharacterObject),
            (status = 400, description = "Invalid video character update payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested video character was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to update the video character.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_character_update() {}

#[utoipa::path(
        post,
        path = "/v1/videos/{video_id}/extend",
        tag = "videos",
        params(("video_id" = String, Path, description = "Video identifier.")),
        request_body = sdkwork_api_contract_openai::videos::ExtendVideoRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created extended video.", body = sdkwork_api_contract_openai::videos::VideoObject),
            (status = 400, description = "Invalid video extend payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested video was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to extend the video.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_extend() {}
