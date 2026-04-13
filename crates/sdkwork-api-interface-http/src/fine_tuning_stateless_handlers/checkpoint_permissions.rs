use super::*;

pub(super) async fn fine_tuning_checkpoint_permissions_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuned_model_checkpoint): Path<String>,
    ExtractJson(request): ExtractJson<CreateFineTuningCheckpointPermissionsRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningCheckpointPermissions(&fine_tuned_model_checkpoint, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning checkpoint permissions create",
            );
        }
    }

    Json(
        sdkwork_api_app_gateway::create_fine_tuning_checkpoint_permissions(
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .expect("fine tuning checkpoint permissions create"),
    )
    .into_response()
}

pub(super) async fn fine_tuning_checkpoint_permissions_list_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuned_model_checkpoint): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningCheckpointPermissionsList(&fine_tuned_model_checkpoint),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning checkpoint permissions list",
            );
        }
    }

    Json(
        sdkwork_api_app_gateway::list_fine_tuning_checkpoint_permissions(
            request_context.tenant_id(),
            request_context.project_id(),
            &fine_tuned_model_checkpoint,
        )
        .expect("fine tuning checkpoint permissions list"),
    )
    .into_response()
}

pub(super) async fn fine_tuning_checkpoint_permission_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((fine_tuned_model_checkpoint, permission_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningCheckpointPermissionsDelete(
            &fine_tuned_model_checkpoint,
            &permission_id,
        ),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning checkpoint permission delete",
            );
        }
    }

    Json(
        sdkwork_api_app_gateway::delete_fine_tuning_checkpoint_permission(
            request_context.tenant_id(),
            request_context.project_id(),
            &permission_id,
        )
        .expect("fine tuning checkpoint permission delete"),
    )
    .into_response()
}
