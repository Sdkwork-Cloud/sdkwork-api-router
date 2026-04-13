use super::*;

pub(super) async fn fine_tuning_job_events_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningJobsEvents(&fine_tuning_job_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job events");
        }
    }

    Json(
        list_fine_tuning_job_events(
            request_context.tenant_id(),
            request_context.project_id(),
            &fine_tuning_job_id,
        )
        .expect("fine tuning job events"),
    )
    .into_response()
}

pub(super) async fn fine_tuning_job_checkpoints_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningJobsCheckpoints(&fine_tuning_job_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning job checkpoints",
            );
        }
    }

    Json(
        list_fine_tuning_job_checkpoints(
            request_context.tenant_id(),
            request_context.project_id(),
            &fine_tuning_job_id,
        )
        .expect("fine tuning job checkpoints"),
    )
    .into_response()
}
