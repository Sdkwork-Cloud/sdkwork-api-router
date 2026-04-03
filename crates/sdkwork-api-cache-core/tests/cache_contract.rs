use anyhow::Result;
use async_trait::async_trait;
use sdkwork_api_cache_core::{
    cache_get_or_insert_with, CacheBackendKind, CacheDriverFactory, CacheDriverRegistry,
    CacheEntry, CacheRuntimeStores, CacheStore, CacheTag, DistributedLockStore,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default)]
struct RecordingCacheStore {
    entries: Mutex<HashMap<String, CacheEntry>>,
    tags: Mutex<HashMap<String, HashSet<String>>>,
    loader_calls: Mutex<u64>,
}

#[async_trait]
impl CacheStore for RecordingCacheStore {
    async fn get(&self, namespace: &str, key: &str) -> Result<Option<CacheEntry>> {
        let composite = format!("{namespace}:{key}");
        Ok(self.entries.lock().await.get(&composite).cloned())
    }

    async fn put(
        &self,
        namespace: &str,
        key: &str,
        value: Vec<u8>,
        ttl_ms: Option<u64>,
        tags: &[CacheTag],
    ) -> Result<()> {
        let composite = format!("{namespace}:{key}");
        self.entries.lock().await.insert(
            composite.clone(),
            CacheEntry::new(value).with_ttl_ms_option(ttl_ms),
        );
        let mut tag_index = self.tags.lock().await;
        for tag in tags {
            tag_index
                .entry(tag.value().to_owned())
                .or_default()
                .insert(composite.clone());
        }
        Ok(())
    }

    async fn delete(&self, namespace: &str, key: &str) -> Result<bool> {
        let composite = format!("{namespace}:{key}");
        Ok(self.entries.lock().await.remove(&composite).is_some())
    }

    async fn invalidate_tag(&self, namespace: &str, tag: &str) -> Result<u64> {
        let tagged = self.tags.lock().await.remove(tag).unwrap_or_default();
        let mut entries = self.entries.lock().await;
        let mut removed = 0_u64;
        for composite in tagged {
            if composite.starts_with(&format!("{namespace}:"))
                && entries.remove(&composite).is_some()
            {
                removed += 1;
            }
        }
        Ok(removed)
    }
}

#[async_trait]
impl DistributedLockStore for RecordingCacheStore {
    async fn try_acquire_lock(&self, _scope: &str, _owner: &str, _ttl_ms: u64) -> Result<bool> {
        Ok(true)
    }

    async fn release_lock(&self, _scope: &str, _owner: &str) -> Result<bool> {
        Ok(true)
    }
}

struct RecordingCacheDriverFactory;

#[async_trait]
impl CacheDriverFactory for RecordingCacheDriverFactory {
    fn backend_kind(&self) -> CacheBackendKind {
        CacheBackendKind::Memory
    }

    fn driver_name(&self) -> &'static str {
        "recording-cache"
    }

    async fn build(&self, _cache_url: Option<&str>) -> Result<CacheRuntimeStores> {
        let store = Arc::new(RecordingCacheStore::default());
        Ok(CacheRuntimeStores::new(store.clone(), store))
    }
}

#[tokio::test]
async fn cache_get_or_insert_with_populates_once_and_reuses_cached_value() {
    let cache = Arc::new(RecordingCacheStore::default());

    let first = cache_get_or_insert_with(
        cache.as_ref(),
        "routing",
        "provider-selection",
        Some(5_000),
        &[CacheTag::new("policy:default")],
        || {
            let cache = cache.clone();
            async move {
                *cache.loader_calls.lock().await += 1;
                Ok::<_, anyhow::Error>(b"provider-a".to_vec())
            }
        },
    )
    .await
    .unwrap();

    let second = cache_get_or_insert_with(
        cache.as_ref(),
        "routing",
        "provider-selection",
        Some(5_000),
        &[CacheTag::new("policy:default")],
        || async move { Ok::<_, anyhow::Error>(b"provider-b".to_vec()) },
    )
    .await
    .unwrap();

    assert_eq!(first, b"provider-a".to_vec());
    assert_eq!(second, b"provider-a".to_vec());
    assert_eq!(*cache.loader_calls.lock().await, 1);
}

#[test]
fn cache_backend_kind_parses_memory_and_redis() {
    assert_eq!(
        CacheBackendKind::parse("memory").unwrap(),
        CacheBackendKind::Memory
    );
    assert_eq!(
        CacheBackendKind::parse("redis").unwrap(),
        CacheBackendKind::Redis
    );
}

#[test]
fn cache_backend_kind_reports_shared_cache_coherence_support() {
    assert!(!CacheBackendKind::Memory.supports_shared_cache_coherence());
    assert!(CacheBackendKind::Redis.supports_shared_cache_coherence());
}

#[tokio::test]
async fn cache_driver_registry_resolves_registered_backend_and_builds_runtime_stores() {
    let registry = CacheDriverRegistry::new().with_factory(RecordingCacheDriverFactory);

    let driver = registry
        .resolve(CacheBackendKind::Memory)
        .expect("memory backend should resolve");
    let stores = driver.build(None).await.unwrap();

    stores
        .cache_store()
        .put(
            "routing",
            "selection",
            b"provider-a".to_vec(),
            Some(5_000),
            &[CacheTag::new("policy:default")],
        )
        .await
        .unwrap();
    let cached = stores
        .cache_store()
        .get("routing", "selection")
        .await
        .unwrap()
        .expect("cached entry");

    assert_eq!(driver.driver_name(), "recording-cache");
    assert_eq!(cached.value(), b"provider-a");
}
