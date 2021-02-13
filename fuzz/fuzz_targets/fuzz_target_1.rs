#![no_main]
use applejack::TrieNode;
use lazy_static::lazy_static;
use libfuzzer_sys::fuzz_target;
use std::sync::Mutex;

lazy_static! {
    static ref TRIE: Mutex<TrieNode<usize>> = Mutex::new(TrieNode::default());
}

fuzz_target!(|data: &[u8]| {
    let mut node = TRIE.lock().unwrap();
    node.insert(data, data.len());
    assert!(node.exists(data));
});
