use sdkwork_api_app_gateway::{
    create_webhook, delete_webhook, get_webhook, list_webhooks, update_webhook,
};

#[test]
fn returns_webhook_object() {
    let response = create_webhook(
        "tenant-1",
        "project-1",
        "https://example.com/webhook",
        &["response.completed".to_owned()],
    )
    .unwrap();
    assert_eq!(response.object, "webhook_endpoint");
    assert_eq!(response.url, "https://example.com/webhook");
}

#[test]
fn lists_webhook_objects() {
    let response = list_webhooks("tenant-1", "project-1").unwrap();
    assert_eq!(response.object, "list");
    assert_eq!(response.data[0].object, "webhook_endpoint");
}

#[test]
fn retrieves_webhook_object() {
    let response = get_webhook("tenant-1", "project-1", "wh_1").unwrap();
    assert_eq!(response.id, "wh_1");
    assert_eq!(response.object, "webhook_endpoint");
}

#[test]
fn updates_webhook_object() {
    let response = update_webhook(
        "tenant-1",
        "project-1",
        "wh_1",
        "https://example.com/webhook/v2",
    )
    .unwrap();
    assert_eq!(response.id, "wh_1");
    assert_eq!(response.url, "https://example.com/webhook/v2");
}

#[test]
fn deletes_webhook_object() {
    let response = delete_webhook("tenant-1", "project-1", "wh_1").unwrap();
    assert_eq!(response.id, "wh_1");
    assert!(response.deleted);
}
