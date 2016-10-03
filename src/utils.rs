

use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use time;


define_encode_set! {
    pub MY_ENCODE_SET = [DEFAULT_ENCODE_SET] | {' ', '"', '#', '<', '>', '/', ':', ';', '=', '@', '[', '\\', ']', '^', '|', '&'}
}


pub fn encode_url(url:&str) -> String {
    utf8_percent_encode(url, MY_ENCODE_SET).collect()
}


/**
 * Get current time millis.
 */
pub fn current_time_millis() -> u64 {
    let ts = time::get_time();
    ( (ts.sec * 1000) as f64 + (ts.nsec as f64 / 1000.0 / 1000.0) ) as u64
}
