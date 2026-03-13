use sdkwork_api_contract_openai::webhooks::{
    CreateWebhookRequest, DeleteWebhookResponse, ListWebhooksResponse, UpdateWebhookRequest,
    WebhookObject,
};

#[test]
fn serializes_webhook_resource_contracts() {
    let request =
        CreateWebhookRequest::new("https://example.com/webhook", vec!["response.completed"]);
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["url"], "https://example.com/webhook");
    assert_eq!(request_json["events"][0], "response.completed");

    let update = UpdateWebhookRequest::new("https://example.com/webhook/v2");
    let update_json = serde_json::to_value(update).unwrap();
    assert_eq!(update_json["url"], "https://example.com/webhook/v2");

    let webhook = WebhookObject::new("wh_1", "https://example.com/webhook");
    let webhook_json = serde_json::to_value(webhook).unwrap();
    assert_eq!(webhook_json["object"], "webhook_endpoint");

    let list = ListWebhooksResponse::new(vec![WebhookObject::new(
        "wh_1",
        "https://example.com/webhook",
    )]);
    let list_json = serde_json::to_value(list).unwrap();
    assert_eq!(list_json["object"], "list");
    assert_eq!(list_json["data"][0]["id"], "wh_1");

    let deleted = DeleteWebhookResponse::deleted("wh_1");
    let deleted_json = serde_json::to_value(deleted).unwrap();
    assert_eq!(deleted_json["object"], "webhook_endpoint.deleted");
    assert_eq!(deleted_json["deleted"], true);
}
