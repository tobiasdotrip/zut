use crate::trash::{self, PurgeStats, parse_duration};

pub fn run_autopurge(auto_purge_after: &str) -> Option<PurgeStats> {
    let duration = parse_duration(auto_purge_after).ok()?;
    trash::purge_older_than(duration)
        .ok()
        .filter(|s| s.count > 0)
}
