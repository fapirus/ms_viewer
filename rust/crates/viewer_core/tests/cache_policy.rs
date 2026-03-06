use viewer_core::cache::{compute_prefetch_window, CachePolicy, LruPageCache};

#[test]
fn default_cache_policy_matches_documented_values() {
    let policy = CachePolicy::default();

    assert_eq!(policy.render_window, 3);
    assert_eq!(policy.prefetch_radius, 1);
}

#[test]
fn lru_page_cache_evicts_oldest_entry() {
    let mut cache = LruPageCache::new(3);
    cache.touch(1);
    cache.touch(2);
    cache.touch(3);
    cache.touch(4);

    assert_eq!(cache.pages(), vec![4, 3, 2]);
}

#[test]
fn prefetch_window_includes_adjacent_pages() {
    assert_eq!(compute_prefetch_window(5, 10, 1), vec![4, 5, 6]);
    assert_eq!(compute_prefetch_window(0, 10, 1), vec![0, 1]);
}
