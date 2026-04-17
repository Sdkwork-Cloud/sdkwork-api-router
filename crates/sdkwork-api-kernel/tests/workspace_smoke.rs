use sdkwork_api_kernel::workspace_name;

#[test]
fn exposes_workspace_name() {
    assert_eq!(workspace_name(), "sdkwork-api-router");
}
