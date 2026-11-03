//! Fixed-window hit counters in Redis, shared by rate limits and lockouts.

use crate::RedisConn;

/// Adds one hit to `key` and returns the count with the seconds left in its
/// window.
///
/// `INCR` and `EXPIRE ... NX` run in one transaction, so a counter never
/// outlives its window: a separate `EXPIRE` that failed after the `INCR`
/// used to leave a limit that never reset, and `NX` also heals a key left
/// without a TTL without stretching a live window.
pub async fn hit_window(
    redis: &mut RedisConn,
    key: &str,
    window_secs: u64,
) -> Result<(i64, i64), redis::RedisError> {
    redis::pipe()
        .atomic()
        .incr(key, 1)
        .cmd("EXPIRE")
        .arg(key)
        .arg(window_secs)
        .arg("NX")
        .ignore()
        .ttl(key)
        .query_async(redis)
        .await
}

/// Takes back one hit from each key that still exists, for work refused
/// before it happened. A key that already expired stays gone: `DECR` would
/// recreate it without a TTL.
pub async fn release_hits(redis: &mut RedisConn, keys: &[String]) -> Result<(), redis::RedisError> {
    redis::Script::new(
        "for _, key in ipairs(KEYS) do \
           if redis.call('EXISTS', key) == 1 then redis.call('DECR', key) end \
         end \
         return 0",
    )
    .key(keys)
    .invoke_async::<i64>(redis)
    .await
    .map(|_| ())
}
