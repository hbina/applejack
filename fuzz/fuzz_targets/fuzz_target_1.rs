#![no_main]
use applejack::TrieNode;
use lazy_static::lazy_static;
use libfuzzer_sys::fuzz_target;
use std::sync::Mutex;

lazy_static! {
    static ref TRIE: Mutex<TrieNode<()>> = Mutex::new(TrieNode::default());
}

fuzz_target!(|data: &[u8]| {
    let mut node = TRIE.lock().unwrap();
    if data.len() == 0 {
        node.insert(data, ());
        assert!(node.exists(data));
        node.remove(data);
        assert!(node.exists(data));
    } else {
        node.insert(data, ());
        assert!(node.exists(data));
        node.remove(data);
        assert!(!node.exists(data));
    }
});
