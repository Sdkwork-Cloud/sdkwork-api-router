use super::*;

pub(super) async fn thread_and_run_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateThreadAndRunRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ThreadsRuns(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread and run");
        }
    }

    Json(
        create_thread_and_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.assistant_id,
        )
        .expect("thread and run create"),
    )
    .into_response()
}

pub(super) async fn thread_runs_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateRunRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRuns(&thread_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run");
        }
    }

    Json(
        create_thread_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &request.assistant_id,
            request.model.as_deref(),
        )
        .expect("thread run create"),
    )
    .into_response()
}
