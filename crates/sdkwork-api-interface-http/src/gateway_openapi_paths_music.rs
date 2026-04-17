use super::*;

#[utoipa::path(
        get,
        path = "/v1/music",
        operation_id = "music_openai_list",
        tag = "music.openai",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible music tracks.", body = sdkwork_api_contract_openai::music::MusicTracksResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load music tracks.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn music_list() {}

#[utoipa::path(
        post,
        path = "/v1/music",
        operation_id = "music_openai_create",
        tag = "music.openai",
        request_body = sdkwork_api_contract_openai::music::CreateMusicRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created music track.", body = sdkwork_api_contract_openai::music::MusicObject),
            (status = 400, description = "Invalid music payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the music track.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn music_create() {}

#[utoipa::path(
        get,
        path = "/v1/music/{music_id}",
        operation_id = "music_openai_get",
        tag = "music.openai",
        params(("music_id" = String, Path, description = "Music track identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible music track metadata.", body = sdkwork_api_contract_openai::music::MusicObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested music track was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the music track.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn music_get() {}

#[utoipa::path(
        delete,
        path = "/v1/music/{music_id}",
        operation_id = "music_openai_delete",
        tag = "music.openai",
        params(("music_id" = String, Path, description = "Music track identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted music track.", body = sdkwork_api_contract_openai::music::DeleteMusicResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested music track was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the music track.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn music_delete() {}

#[utoipa::path(
        get,
        path = "/v1/music/{music_id}/content",
        operation_id = "music_openai_content_get",
        tag = "music.openai",
        params(("music_id" = String, Path, description = "Music track identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Binary music content stream."),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested music track was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the music content.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn music_content() {}

#[utoipa::path(
        post,
        path = "/v1/music/lyrics",
        operation_id = "music_openai_lyrics_create",
        tag = "music.openai",
        request_body = sdkwork_api_contract_openai::music::CreateMusicLyricsRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created music lyrics.", body = sdkwork_api_contract_openai::music::MusicLyricsObject),
            (status = 400, description = "Invalid music lyrics payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the music lyrics.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn music_lyrics() {}
