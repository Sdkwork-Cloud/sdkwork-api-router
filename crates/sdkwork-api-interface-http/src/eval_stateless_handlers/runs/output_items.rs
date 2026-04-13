use super::*;

pub(super) async fn eval_run_output_items_list_handler(
    request_context: StatelessGatewayRequest,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRunOutputItemsList(&eval_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream eval run output items list",
            );
        }
    }

    Json(
        sdkwork_api_app_gateway::list_eval_run_output_items(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
            &run_id,
        )
        .expect("eval run output items list"),
    )
    .into_response()
}

pub(super) async fn eval_run_output_item_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((eval_id, run_id, output_item_id)): Path<(String, String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRunOutputItemsRetrieve(&eval_id, &run_id, &output_item_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream eval run output item retrieve",
            );
        }
    }

    Json(
        sdkwork_api_app_gateway::get_eval_run_output_item(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
            &run_id,
            &output_item_id,
        )
        .expect("eval run output item retrieve"),
    )
    .into_response()
}
