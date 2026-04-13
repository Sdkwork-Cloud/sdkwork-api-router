use super::*;

pub(super) async fn thread_runs_list_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsList(&thread_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread runs list");
        }
    }

    Json(
        list_thread_runs(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
        )
        .expect("thread runs list"),
    )
    .into_response()
}

pub(super) async fn thread_run_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsRetrieve(&thread_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run retrieve");
        }
    }

    Json(
        get_thread_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &run_id,
        )
        .expect("thread run"),
    )
    .into_response()
}

pub(super) async fn thread_run_update_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<UpdateRunRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsUpdate(&thread_id, &run_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run update");
        }
    }

    Json(
        update_thread_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &run_id,
        )
        .expect("thread run update"),
    )
    .into_response()
}

pub(super) async fn thread_run_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsCancel(&thread_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run cancel");
        }
    }

    Json(
        cancel_thread_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &run_id,
        )
        .expect("thread run cancel"),
    )
    .into_response()
}
