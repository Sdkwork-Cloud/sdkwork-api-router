use sdkwork_api_app_gateway::{
    cancel_vector_store_file_batch, create_vector_store_file_batch, get_vector_store_file_batch,
    list_vector_store_file_batch_files,
};

#[test]
fn returns_vector_store_file_batch_object() {
    let response =
        create_vector_store_file_batch("tenant-1", "project-1", "vs_1", &["file_1"]).unwrap();
    assert_eq!(response.id, "vsfb_1");
    assert_eq!(response.object, "vector_store.file_batch");
}

#[test]
fn retrieves_vector_store_file_batch_object() {
    let response = get_vector_store_file_batch("tenant-1", "project-1", "vs_1", "vsfb_1").unwrap();
    assert_eq!(response.id, "vsfb_1");
    assert_eq!(response.object, "vector_store.file_batch");
}

#[test]
fn cancels_vector_store_file_batch_object() {
    let response =
        cancel_vector_store_file_batch("tenant-1", "project-1", "vs_1", "vsfb_1").unwrap();
    assert_eq!(response.id, "vsfb_1");
    assert_eq!(response.status, "cancelled");
}

#[test]
fn lists_vector_store_file_batch_files() {
    let response =
        list_vector_store_file_batch_files("tenant-1", "project-1", "vs_1", "vsfb_1").unwrap();
    assert_eq!(response.object, "list");
    assert_eq!(response.data[0].object, "vector_store.file");
}
