#![no_main]
use applejack::Rax;
use lazy_static::lazy_static;
use libfuzzer_sys::fuzz_target;
use std::sync::Mutex;

lazy_static! {
    static ref TRIE: Mutex<Rax<()>> = Mutex::new(Rax::default());
}

fuzz_target!(|data: &[u8]| {
    let mut node = TRIE.lock().unwrap();
    node.insert(data, ());
    assert!(node.exists(data));
    node.remove(data);
    assert!(!node.exists(data));
});
