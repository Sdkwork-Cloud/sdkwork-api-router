use super::*;

pub(super) async fn thread_run_steps_list_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunStepsList(&thread_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run steps list");
        }
    }

    Json(
        list_thread_run_steps(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &run_id,
        )
        .expect("thread run steps"),
    )
    .into_response()
}

pub(super) async fn thread_run_step_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id, step_id)): Path<(String, String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunStepsRetrieve(&thread_id, &run_id, &step_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream thread run step retrieve",
            );
        }
    }

    Json(
        get_thread_run_step(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &run_id,
            &step_id,
        )
        .expect("thread run step"),
    )
    .into_response()
}
