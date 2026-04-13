use super::*;

pub(super) async fn evals_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateEvalRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Evals(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval");
        }
    }

    Json(
        create_eval(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.name,
        )
        .expect("eval"),
    )
    .into_response()
}

pub(super) async fn evals_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::EvalsList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream evals list");
        }
    }

    Json(list_evals(request_context.tenant_id(), request_context.project_id()).expect("eval list"))
        .into_response()
}

pub(super) async fn eval_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::EvalsRetrieve(&eval_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval retrieve");
        }
    }

    Json(
        get_eval(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
        )
        .expect("eval retrieve"),
    )
    .into_response()
}

pub(super) async fn eval_update_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateEvalRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalsUpdate(&eval_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval update");
        }
    }

    Json(
        update_eval(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
            &request,
        )
        .expect("eval update"),
    )
    .into_response()
}

pub(super) async fn eval_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::EvalsDelete(&eval_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval delete");
        }
    }

    Json(
        delete_eval(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
        )
        .expect("eval delete"),
    )
    .into_response()
}
