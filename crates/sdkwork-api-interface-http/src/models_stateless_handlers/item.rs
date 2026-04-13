use super::*;

pub(super) async fn model_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(model_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ModelsRetrieve(&model_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream model");
        }
    }

    Json(
        get_model(
            request_context.tenant_id(),
            request_context.project_id(),
            &model_id,
        )
        .expect("model response"),
    )
    .into_response()
}

pub(super) async fn model_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(model_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ModelsDelete(&model_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream model delete");
        }
    }

    Json(
        delete_model(
            request_context.tenant_id(),
            request_context.project_id(),
            &model_id,
        )
        .expect("model delete response"),
    )
    .into_response()
}
