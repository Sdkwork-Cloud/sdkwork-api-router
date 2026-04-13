use super::*;

pub(super) async fn batch_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(batch_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::BatchesRetrieve(&batch_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batch retrieve");
        }
    }

    Json(
        get_batch(
            request_context.tenant_id(),
            request_context.project_id(),
            &batch_id,
        )
        .expect("batch retrieve"),
    )
    .into_response()
}

pub(super) async fn batch_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path(batch_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::BatchesCancel(&batch_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batch cancel");
        }
    }

    Json(
        cancel_batch(
            request_context.tenant_id(),
            request_context.project_id(),
            &batch_id,
        )
        .expect("batch cancel"),
    )
    .into_response()
}
