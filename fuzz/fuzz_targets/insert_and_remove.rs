#![no_main]
use applejack::Rax;
use lazy_static::lazy_static;
use libfuzzer_sys::arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use rand::distributions::{Distribution, WeightedIndex};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Arbitrary, Debug)]
struct KeyValue(Vec<u8>, u8);

lazy_static! {
    static ref TRIE: Mutex<(Rax<u8>, HashMap<Vec<u8>, u8>)> =
        Mutex::new((Rax::default(), HashMap::default()));
}

fuzz_target!(|data: KeyValue| {
    let mut rng = rand::thread_rng();
    let mut trie = TRIE.lock().unwrap();
    if rand::random() {
        println!("inserting:{:?}", data);
        trie.0.insert(data.0.as_ref(), data.1.clone());
        trie.1.insert(data.0.clone(), data.1.clone());
        assert!(trie.0.exists(data.0.as_ref()));
    } else {
        println!("removing");
        println!("trie:{:#?}", trie);
        if !trie.1.is_empty() {
            let choices = trie
                .1
                .keys()
                .map(|key| (key.clone(), 1))
                .collect::<Vec<(Vec<u8>, usize)>>();
            let choice = WeightedIndex::new(choices.iter().map(|v| v.1))
                .unwrap()
                .sample(&mut rng);
            let picked_key = &choices[choice];
            println!("picked_key:{:#?}", choice);
            let removed_value = trie.0.remove(&picked_key.0.as_ref()).unwrap();
            let stored_value = trie.1.remove(&picked_key.0).unwrap();
            assert_eq!(stored_value, removed_value);
        }
    }
});
