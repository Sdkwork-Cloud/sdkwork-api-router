use super::*;

pub(super) async fn image_generations_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateImageRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ImagesGenerations(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream image generation");
        }
    }

    Json(
        create_image_generation(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("image generation"),
    )
    .into_response()
}

pub(super) async fn image_edits_handler(
    request_context: StatelessGatewayRequest,
    multipart: Multipart,
) -> Response {
    match parse_image_edit_request(multipart).await {
        Ok(request) => {
            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::ImagesEdits(&request),
            )
            .await
            {
                Ok(Some(response)) => return Json(response).into_response(),
                Ok(None) => {}
                Err(_) => {
                    return bad_gateway_openai_response("failed to relay upstream image edit");
                }
            }

            Json(
                create_image_edit(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request,
                )
                .expect("image edit"),
            )
            .into_response()
        }
        Err(response) => response,
    }
}

pub(super) async fn image_variations_handler(
    request_context: StatelessGatewayRequest,
    multipart: Multipart,
) -> Response {
    match parse_image_variation_request(multipart).await {
        Ok(request) => {
            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::ImagesVariations(&request),
            )
            .await
            {
                Ok(Some(response)) => return Json(response).into_response(),
                Ok(None) => {}
                Err(_) => {
                    return bad_gateway_openai_response("failed to relay upstream image variation");
                }
            }

            Json(
                create_image_variation(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request,
                )
                .expect("image variation"),
            )
            .into_response()
        }
        Err(response) => response,
    }
}
