use sdkwork_api_app_gateway::{
    create_conversation, create_conversation_items, delete_conversation, delete_conversation_item,
    get_conversation, get_conversation_item, list_conversation_items, list_conversations,
    update_conversation,
};

#[test]
fn returns_conversation_object() {
    let response = create_conversation("tenant-1", "project-1").unwrap();
    assert_eq!(response.object, "conversation");
}

#[test]
fn lists_conversation_objects() {
    let response = list_conversations("tenant-1", "project-1").unwrap();
    assert_eq!(response.object, "list");
    assert_eq!(response.data[0].object, "conversation");
}

#[test]
fn retrieves_conversation_object() {
    let response = get_conversation("tenant-1", "project-1", "conv_1").unwrap();
    assert_eq!(response.id, "conv_1");
}

#[test]
fn updates_conversation_object() {
    let response = update_conversation(
        "tenant-1",
        "project-1",
        "conv_1",
        serde_json::json!({"workspace":"next"}),
    )
    .unwrap();
    assert_eq!(
        response.metadata,
        Some(serde_json::json!({"workspace":"next"}))
    );
}

#[test]
fn deletes_conversation_object() {
    let response = delete_conversation("tenant-1", "project-1", "conv_1").unwrap();
    assert_eq!(response.id, "conv_1");
    assert!(response.deleted);
}

#[test]
fn creates_conversation_items() {
    let response = create_conversation_items("tenant-1", "project-1", "conv_1").unwrap();
    assert_eq!(response.object, "list");
    assert_eq!(response.data[0].object, "conversation.item");
}

#[test]
fn lists_conversation_items() {
    let response = list_conversation_items("tenant-1", "project-1", "conv_1").unwrap();
    assert_eq!(response.object, "list");
    assert_eq!(response.data[0].id, "item_1");
}

#[test]
fn retrieves_conversation_item() {
    let response = get_conversation_item("tenant-1", "project-1", "conv_1", "item_1").unwrap();
    assert_eq!(response.id, "item_1");
}

#[test]
fn deletes_conversation_item() {
    let response = delete_conversation_item("tenant-1", "project-1", "conv_1", "item_1").unwrap();
    assert_eq!(response.id, "item_1");
    assert!(response.deleted);
}
