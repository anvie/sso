

use rand;
use rand::Rng;

pub fn generate() -> String {
    rand::thread_rng()
        .gen_ascii_chars()
        .take(50)
        .collect()
}
