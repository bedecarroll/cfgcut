#![no_main]

use cfgcut::fuzz_parse;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        fuzz_parse(text);
    }
});
