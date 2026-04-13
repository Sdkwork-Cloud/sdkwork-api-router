use super::*;

pub(super) async fn video_character_retrieve_canonical_handler(
    request_context: StatelessGatewayRequest,
    Path(character_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideoCharactersCanonicalRetrieve(&character_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream video character retrieve",
            );
        }
    }

    Json(
        sdkwork_api_app_gateway::get_video_character_canonical(
            request_context.tenant_id(),
            request_context.project_id(),
            &character_id,
        )
        .expect("video character canonical retrieve"),
    )
    .into_response()
}
