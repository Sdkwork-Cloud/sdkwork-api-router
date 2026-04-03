use sdkwork_api_cache_core::{CacheStore, CacheTag};
use sdkwork_api_cache_memory::MemoryCacheStore;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn memory_cache_round_trips_values_and_respects_ttl() {
    let cache = MemoryCacheStore::default();

    cache
        .put(
            "routing",
            "candidate",
            b"provider-a".to_vec(),
            Some(25),
            &[CacheTag::new("policy:default")],
        )
        .await
        .unwrap();

    let immediate = cache.get("routing", "candidate").await.unwrap().unwrap();
    assert_eq!(immediate.value(), b"provider-a");

    sleep(Duration::from_millis(40)).await;

    let expired = cache.get("routing", "candidate").await.unwrap();
    assert!(expired.is_none());
}

#[tokio::test]
async fn memory_cache_invalidates_entries_by_tag() {
    let cache = MemoryCacheStore::default();

    cache
        .put(
            "routing",
            "candidate-a",
            b"provider-a".to_vec(),
            None,
            &[CacheTag::new("policy:default")],
        )
        .await
        .unwrap();
    cache
        .put(
            "routing",
            "candidate-b",
            b"provider-b".to_vec(),
            None,
            &[CacheTag::new("policy:default")],
        )
        .await
        .unwrap();

    let removed = cache
        .invalidate_tag("routing", "policy:default")
        .await
        .unwrap();
    assert_eq!(removed, 2);
    assert!(cache.get("routing", "candidate-a").await.unwrap().is_none());
    assert!(cache.get("routing", "candidate-b").await.unwrap().is_none());
}
