use applejack::TrieNode;
use smallvec::smallvec;

#[test]
fn general_tests() {
    let mut trie = TrieNode::new();
    trie.insert(&[0, 1, 2]);
    trie.insert(&[0, 1, 2, 3, 4]);
    trie.insert(&[0, 1, 3]);
    trie.insert(&[]);
    trie.insert(&[]);
    assert!(trie.exists(&[0, 1, 2]));
    assert!(trie.exists(&[0, 1, 2, 3, 4]));
    assert!(trie.exists(&[0, 1, 3]));
    assert!(trie.exists(&[]));
    assert!(!trie.exists(&[1, 2, 3]));
    assert_eq!(
        trie,
        TrieNode {
            prefix: smallvec![],
            branches: vec![TrieNode {
                prefix: smallvec![0, 1],
                branches: vec![
                    TrieNode {
                        prefix: smallvec![2],
                        branches: vec![TrieNode {
                            prefix: smallvec![3, 4],
                            branches: vec![]
                        },]
                    },
                    TrieNode {
                        prefix: smallvec![3],
                        branches: vec![]
                    }
                ]
            },]
        }
    )
}

#[test]
fn insert_empty() {
    let mut trie = TrieNode::new();
    trie.insert(&[0, 1, 2]);
    trie.insert(&[]);
    trie.insert(&[0, 1, 2, 3, 4]);
    trie.insert(&[0, 1, 2, 3, 4, 5, 6]);
    assert!(trie.exists(&[0, 1, 2]));
    assert!(trie.exists(&[]));
    assert!(!trie.exists(&[0, 1, 2, 3]));
    assert!(!trie.exists(&[0, 1]));
    assert_eq!(
        trie,
        TrieNode {
            prefix: smallvec![],
            branches: vec![TrieNode {
                prefix: smallvec![0, 1, 2],
                branches: vec![TrieNode {
                    prefix: smallvec![3, 4],
                    branches: vec![TrieNode {
                        prefix: smallvec![5, 6],
                        branches: vec![]
                    }]
                }]
            },]
        }
    )
}

#[test]
fn insert_very_different_strings() {
    let mut trie = TrieNode::new();
    trie.insert(&[0, 1, 2, 3]);
    trie.insert(&[4, 5, 6, 7]);
    assert_eq!(
        trie,
        TrieNode {
            prefix: smallvec![],
            branches: vec![
                TrieNode {
                    prefix: smallvec![0, 1, 2, 3],
                    branches: vec![]
                },
                TrieNode {
                    prefix: smallvec![4, 5, 6, 7],
                    branches: vec![]
                }
            ]
        }
    )
}

#[test]
fn get_something_that_exist() {
    let mut trie = TrieNode::new();
    trie.insert(&[0, 1, 2, 3]);
    trie.insert(&[4, 5, 6, 7]);
    assert_eq!(
        trie,
        TrieNode {
            prefix: smallvec![],
            branches: vec![
                TrieNode {
                    prefix: smallvec![0, 1, 2, 3],
                    branches: vec![]
                },
                TrieNode {
                    prefix: smallvec![4, 5, 6, 7],
                    branches: vec![]
                }
            ]
        }
    );
    assert!(trie.exists(&[0, 1, 2, 3]));
}

#[test]
fn initialize_with_something_big() {
    let mut trie = TrieNode::with_key(&[0, 1, 2, 3]);
    trie.insert(&[0, 1, 2, 3, 4]);
    assert_eq!(
        trie,
        TrieNode {
            prefix: smallvec![0, 1, 2, 3],
            branches: vec![TrieNode {
                prefix: smallvec![4],
                branches: vec![]
            },]
        }
    );
    assert!(trie.exists(&[0, 1, 2, 3]));
}

#[test]
fn get_empty_exists() {
    let trie = TrieNode::new();
    assert!(trie.exists(&[]));
}

#[test]
fn get_nested_exists() {
    let mut trie = TrieNode::new();
    trie.insert(&[0, 1, 2]);
    trie.insert(&[]);
    trie.insert(&[0, 1, 2, 3, 4]);
    trie.insert(&[0, 1, 2, 3, 4, 5, 6]);
    assert_eq!(
        trie,
        TrieNode {
            prefix: smallvec![],
            branches: vec![TrieNode {
                prefix: smallvec![0, 1, 2],
                branches: vec![TrieNode {
                    prefix: smallvec![3, 4],
                    branches: vec![TrieNode {
                        prefix: smallvec![5, 6],
                        branches: vec![]
                    }]
                }]
            },]
        }
    );
    assert!(!trie.exists(&[0, 1, 2, 3]));
    assert!(trie.exists(&[0, 1, 2, 3, 4]));
    assert!(!trie.exists(&[0, 1, 2, 3, 4, 5]));
    assert!(trie.exists(&[0, 1, 2, 3, 4, 5, 6]));
}

#[test]
fn assert_size_of_node() {
    assert_eq!(56, std::mem::size_of::<TrieNode>());
}
