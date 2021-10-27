#![no_main]
use applejack::Rax;

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
enum Operation {
    Insert(Vec<u8>, u8),
    Remove(Vec<u8>),
    Get(Vec<u8>),
}

fuzz_target!(|data: Vec<Operation>| {
    let mut table = std::collections::HashMap::new();
    let mut rax = Rax::default();
    for x in &data {
        match x {
            Operation::Insert(key, value) => {
                rax.insert(key, *value);
                table.insert(key, *value);
            }
            Operation::Remove(key) => {
                assert_eq!(rax.remove(key), table.remove(key));
            }
            Operation::Get(key) => {
                assert_eq!(rax.get(key), table.get(key))
            }
        };
    }
});
