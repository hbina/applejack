#![no_main]
use applejack::Rax;

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
enum KeyValue {
    Insert(Vec<u8>, u8),
    Remove(Vec<u8>),
}

fuzz_target!(|data: Vec<KeyValue>| {
    let mut rax = Rax::default();
    for x in &data {
        match x {
            KeyValue::Insert(key, value) => rax.insert(key, *value),
            KeyValue::Remove(key) => {
                rax.remove(key);
            }
        };
    }
});
