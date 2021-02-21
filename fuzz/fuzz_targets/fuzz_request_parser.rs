#![no_main]
use libfuzzer_sys::fuzz_target;

use dray::request::Request;
use std::convert::TryFrom;

fuzz_target!(|data: &[u8]| {
    // The result is acceptable as long as a panic does not result
    match Request::try_from(data) {
        _ => ()
    };
});
