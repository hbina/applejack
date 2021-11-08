#![no_main]
use applejack::Rax;

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
enum Operation {
    Insert(Vec<u8>, u8),
    Remove(Vec<u8>),
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
        };
    }
    assert_eq!(rax.iter().map(|s| s).collect::<Vec<_>>().len(), table.len());
});
