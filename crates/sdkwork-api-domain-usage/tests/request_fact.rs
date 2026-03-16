use sdkwork_api_domain_usage::UsageRecord;

#[test]
fn usage_record_tracks_project() {
    let usage = UsageRecord::new("project-1", "gpt-4.1", "provider-openai-official");
    assert_eq!(usage.project_id, "project-1");
    assert_eq!(usage.units, 0);
    assert_eq!(usage.amount, 0.0);
}
