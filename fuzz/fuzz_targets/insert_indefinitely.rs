#![no_main]
use applejack::Rax;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: Vec<Vec<u8>>| {
    let mut rax = Rax::default();
    for key in &data {
        rax.insert(key, ());
    }
});
