use super::*;

pub(super) async fn vector_store_file_batches_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateVectorStoreFileBatchRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFileBatches(&vector_store_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store file batch");
        }
    }
    Json(
        create_vector_store_file_batch(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            &request.file_ids,
        )
        .expect("vector store file batch"),
    )
    .into_response()
}

pub(super) async fn vector_store_file_batch_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFileBatchesRetrieve(&vector_store_id, &batch_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file batch retrieve",
            );
        }
    }
    Json(
        get_vector_store_file_batch(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            &batch_id,
        )
        .expect("vector store file batch retrieve"),
    )
    .into_response()
}

pub(super) async fn vector_store_file_batch_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFileBatchesCancel(&vector_store_id, &batch_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file batch cancel",
            );
        }
    }
    Json(
        cancel_vector_store_file_batch(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            &batch_id,
        )
        .expect("vector store file batch cancel"),
    )
    .into_response()
}

pub(super) async fn vector_store_file_batch_files_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFileBatchesListFiles(&vector_store_id, &batch_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file batch files",
            );
        }
    }
    Json(
        list_vector_store_file_batch_files(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            &batch_id,
        )
        .expect("vector store file batch files"),
    )
    .into_response()
}
