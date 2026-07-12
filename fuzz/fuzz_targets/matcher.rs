#![no_main]

use cfgcut::fuzz_matcher;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() < 3 {
        return;
    }
    let control = data[0];
    let payload = &data[1..];
    if payload.len() < 2 {
        return;
    }
    let split = 1 + usize::from(control) % (payload.len() - 1);
    let (pattern_bytes, config_bytes) = payload.split_at(split);
    if pattern_bytes.is_empty() {
        return;
    }
    let pattern = std::string::String::from_utf8_lossy(pattern_bytes);
    let config = std::string::String::from_utf8_lossy(config_bytes);
    fuzz_matcher(&pattern, &config);
});
