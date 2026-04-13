use super::*;

pub(super) async fn thread_run_submit_tool_outputs_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<SubmitToolOutputsRunRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsSubmitToolOutputs(&thread_id, &run_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream thread run submit tool outputs",
            );
        }
    }

    let tool_outputs = request
        .tool_outputs
        .iter()
        .map(|output| (output.tool_call_id.as_str(), output.output.as_str()))
        .collect();
    Json(
        submit_thread_run_tool_outputs(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &run_id,
            tool_outputs,
        )
        .expect("thread run submit tool outputs"),
    )
    .into_response()
}
