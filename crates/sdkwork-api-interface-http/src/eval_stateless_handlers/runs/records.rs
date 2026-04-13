use super::*;

pub(super) async fn eval_runs_list_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::EvalRunsList(&eval_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval runs list");
        }
    }

    Json(
        list_eval_runs(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
        )
        .expect("eval runs list"),
    )
    .into_response()
}

pub(super) async fn eval_runs_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateEvalRunRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRuns(&eval_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval run create");
        }
    }

    Json(
        create_eval_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
            &request,
        )
        .expect("eval run create"),
    )
    .into_response()
}

pub(super) async fn eval_run_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRunsRetrieve(&eval_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval run retrieve");
        }
    }

    Json(
        get_eval_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
            &run_id,
        )
        .expect("eval run retrieve"),
    )
    .into_response()
}

pub(super) async fn eval_run_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRunsDelete(&eval_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval run delete");
        }
    }

    Json(
        sdkwork_api_app_gateway::delete_eval_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
            &run_id,
        )
        .expect("eval run delete"),
    )
    .into_response()
}

pub(super) async fn eval_run_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRunsCancel(&eval_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval run cancel");
        }
    }

    Json(
        sdkwork_api_app_gateway::cancel_eval_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
            &run_id,
        )
        .expect("eval run cancel"),
    )
    .into_response()
}
