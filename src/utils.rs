


use time;

/**
 * Get current time millis.
 */
pub fn current_time_millis() -> u64 {
    let ts = time::get_time();
    ( (ts.sec * 1000) as f64 + (ts.nsec as f64 / 1000.0 / 1000.0) ) as u64
}
