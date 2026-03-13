use sdkwork_api_app_gateway::{
    create_thread, create_thread_message, delete_thread, delete_thread_message, get_thread,
    get_thread_message, list_thread_messages, update_thread, update_thread_message,
};

#[test]
fn returns_thread_object() {
    let response = create_thread("tenant-1", "project-1").unwrap();
    assert_eq!(response.object, "thread");
}

#[test]
fn retrieves_updates_and_deletes_thread_object() {
    let retrieved = get_thread("tenant-1", "project-1", "thread_1").unwrap();
    assert_eq!(retrieved.id, "thread_1");
    assert_eq!(retrieved.object, "thread");

    let updated = update_thread("tenant-1", "project-1", "thread_1").unwrap();
    assert_eq!(updated.id, "thread_1");

    let deleted = delete_thread("tenant-1", "project-1", "thread_1").unwrap();
    assert_eq!(deleted.id, "thread_1");
    assert!(deleted.deleted);
}

#[test]
fn manages_thread_messages() {
    let created =
        create_thread_message("tenant-1", "project-1", "thread_1", "user", "hello").unwrap();
    assert_eq!(created.object, "thread.message");
    assert_eq!(created.thread_id, "thread_1");

    let list = list_thread_messages("tenant-1", "project-1", "thread_1").unwrap();
    assert_eq!(list.object, "list");
    assert_eq!(list.data[0].object, "thread.message");

    let retrieved = get_thread_message("tenant-1", "project-1", "thread_1", "msg_1").unwrap();
    assert_eq!(retrieved.id, "msg_1");

    let updated = update_thread_message("tenant-1", "project-1", "thread_1", "msg_1").unwrap();
    assert_eq!(updated.id, "msg_1");

    let deleted = delete_thread_message("tenant-1", "project-1", "thread_1", "msg_1").unwrap();
    assert_eq!(deleted.id, "msg_1");
    assert!(deleted.deleted);
}
