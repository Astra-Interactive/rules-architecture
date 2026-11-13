use crate::event::{JoinListener, ViewCache};

#[test]
fn given_empty_view_cache_when_player_joins_then_no_entries_are_counted() {
    let join_listener = JoinListener::new(ViewCache::new());

    assert_eq!(0, join_listener.on_join());
}
