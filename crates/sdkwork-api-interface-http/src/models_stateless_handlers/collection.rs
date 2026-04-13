use super::*;

pub(super) async fn list_models_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ModelsList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream model list");
        }
    }

    Json(
        list_models(request_context.tenant_id(), request_context.project_id())
            .expect("models response"),
    )
    .into_response()
}
