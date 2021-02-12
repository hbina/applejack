use minivec::{mini_vec, MiniVec};
use smallvec::{smallvec, SmallVec};

type Key = SmallVec<[u8; 8]>;

#[derive(PartialEq, Debug)]
enum Cut {
    Parent(usize),
    Child(usize),
    BothBegin,
    BothMiddle(usize),
    BothEnd,
}

#[derive(PartialEq, Debug)]
pub struct TrieNode<T> {
    pub(crate) value: Option<T>,
    pub(crate) prefix: Key,
    pub(crate) branches: MiniVec<TrieNode<T>>,
}

impl<T> Default for TrieNode<T> {
    fn default() -> Self {
        TrieNode {
            value: None,
            prefix: smallvec![],
            branches: mini_vec![],
        }
    }
}

impl<T> TrieNode<T> {
    pub fn insert(&mut self, new_key: &[u8], value: T) {
        self.insert_impl(new_key, &mut Some(value));
    }

    fn insert_impl(&mut self, new_key: &[u8], value: &mut Option<T>) -> bool {
        let cut = self.cut_key(new_key);
        match cut {
            Cut::Parent(p) => {
                let drained_value = self.value.take();
                self.value = value.take();
                let drained_key = self.prefix.drain(p..).collect::<Key>();
                let drained_branch = self.branches.drain(..).collect();
                self.branches.push(TrieNode {
                    value: drained_value,
                    prefix: drained_key,
                    branches: drained_branch,
                });
                true
            }
            Cut::Child(c) => {
                let cut_child = &new_key[c..];
                let found = self
                    .branches
                    .iter_mut()
                    .any(|x| x.insert_impl(cut_child, value));
                if !found {
                    self.branches.push(TrieNode {
                        value: value.take(),
                        prefix: Key::from(cut_child),
                        branches: mini_vec![],
                    });
                }
                true
            }
            Cut::BothBegin => return false,
            Cut::BothMiddle(p) => {
                let drained_value = self.value.take();
                let drained_key = self.prefix.drain(p..).collect::<Key>();
                let drained_children = self.branches.drain(..).collect();
                self.branches.push(TrieNode {
                    value: drained_value,
                    prefix: drained_key,
                    branches: drained_children,
                });
                self.branches.push(TrieNode {
                    value: value.take(),
                    prefix: Key::from(&new_key[p..]),
                    branches: mini_vec![],
                });
                true
            }
            Cut::BothEnd => {
                self.value = value.take();
                true
            }
        }
    }

    pub fn exists(&self, key: &[u8]) -> bool {
        match self.cut_key(key) {
            Cut::Parent(_) => false,
            Cut::Child(idx) => self.branches.iter().any(|x| x.exists(&key[idx..])),
            Cut::BothBegin => false,
            Cut::BothMiddle(_) => false,
            Cut::BothEnd => true,
        }
    }

    fn cut_key<'b>(&self, child_key: &'b [u8]) -> Cut {
        let idx = self.prefix.iter().zip(child_key).position(|(l, r)| l != r);
        if let Some(idx) = idx {
            if idx == 0 {
                Cut::BothBegin
            } else {
                Cut::BothMiddle(idx)
            }
        } else {
            let (llen, clen) = (self.prefix.len(), child_key.len());
            match llen.cmp(&clen) {
                std::cmp::Ordering::Less => Cut::Child(llen),
                std::cmp::Ordering::Equal => Cut::BothEnd,
                std::cmp::Ordering::Greater => Cut::Parent(clen),
            }
        }
    }
}

#[test]
fn general_tests() {
    let mut trie = TrieNode::default();
    trie.insert(&[0, 1, 2], ());
    trie.insert(&[0, 1, 2, 3, 4], ());
    trie.insert(&[0, 1, 3], ());
    trie.insert(&[], ());
    trie.insert(&[], ());
    assert!(trie.exists(&[0, 1, 2]));
    assert!(trie.exists(&[0, 1, 2, 3, 4]));
    assert!(trie.exists(&[0, 1, 3]));
    assert!(trie.exists(&[]));
    assert!(!trie.exists(&[1, 2, 3]));
    assert_eq!(
        trie,
        TrieNode {
            value: Some(()),
            prefix: smallvec![],
            branches: mini_vec![TrieNode {
                value: None,
                prefix: smallvec![0, 1],
                branches: mini_vec![
                    TrieNode {
                        value: Some(()),
                        prefix: smallvec![2],
                        branches: mini_vec![TrieNode {
                            value: Some(()),
                            prefix: smallvec![3, 4],
                            branches: mini_vec![]
                        },]
                    },
                    TrieNode {
                        value: Some(()),
                        prefix: smallvec![3],
                        branches: mini_vec![]
                    }
                ]
            },]
        }
    )
}

#[test]
fn insert_empty() {
    let mut trie = TrieNode::default();
    trie.insert(&[0, 1, 2], ());
    trie.insert(&[], ());
    trie.insert(&[0, 1, 2, 3, 4], ());
    trie.insert(&[0, 1, 2, 3, 4, 5, 6], ());
    assert!(trie.exists(&[0, 1, 2]));
    assert!(trie.exists(&[]));
    assert!(!trie.exists(&[0, 1, 2, 3]));
    assert!(!trie.exists(&[0, 1]));
    assert_eq!(
        trie,
        TrieNode {
            value: Some(()),
            prefix: smallvec![],
            branches: mini_vec![TrieNode {
                value: Some(()),
                prefix: smallvec![0, 1, 2],
                branches: mini_vec![TrieNode {
                    value: Some(()),
                    prefix: smallvec![3, 4],
                    branches: mini_vec![TrieNode {
                        value: Some(()),
                        prefix: smallvec![5, 6],
                        branches: mini_vec![]
                    }]
                }]
            },]
        }
    )
}

#[test]
fn insert_very_different_strings() {
    let mut trie = TrieNode::default();
    trie.insert(&[0, 1, 2, 3], ());
    trie.insert(&[4, 5, 6, 7], ());
    assert_eq!(
        trie,
        TrieNode {
            value: None,
            prefix: smallvec![],
            branches: mini_vec![
                TrieNode {
                    value: Some(()),
                    prefix: smallvec![0, 1, 2, 3],
                    branches: mini_vec![]
                },
                TrieNode {
                    value: Some(()),
                    prefix: smallvec![4, 5, 6, 7],
                    branches: mini_vec![]
                }
            ]
        }
    )
}

#[test]
fn get_something_that_exist() {
    let mut trie = TrieNode::default();
    trie.insert(&[0, 1, 2, 3], ());
    trie.insert(&[4, 5, 6, 7], ());
    assert_eq!(
        trie,
        TrieNode {
            value: None,
            prefix: smallvec![],
            branches: mini_vec![
                TrieNode {
                    value: Some(()),
                    prefix: smallvec![0, 1, 2, 3],
                    branches: mini_vec![]
                },
                TrieNode {
                    value: Some(()),
                    prefix: smallvec![4, 5, 6, 7],
                    branches: mini_vec![]
                }
            ]
        }
    );
    assert!(trie.exists(&[0, 1, 2, 3]));
}

#[test]
fn initialize_with_something_big() {
    let mut trie = TrieNode::default();
    trie.insert(&[0, 1, 2, 3], ());
    trie.insert(&[0, 1, 2, 3, 4], ());
    assert_eq!(
        trie,
        TrieNode {
            value: None,
            prefix: smallvec![],
            branches: mini_vec![TrieNode {
                value: Some(()),
                prefix: smallvec![0, 1, 2, 3],
                branches: mini_vec![TrieNode {
                    value: Some(()),
                    prefix: smallvec![4],
                    branches: mini_vec![]
                },]
            }]
        }
    );
    assert!(trie.exists(&[0, 1, 2, 3]));
}

#[test]
fn get_empty_exists() {
    let trie = TrieNode::<()>::default();
    assert!(trie.exists(&[]));
}

#[test]
fn get_nested_exists() {
    let mut trie = TrieNode::default();
    trie.insert(&[0, 1, 2], ());
    trie.insert(&[], ());
    trie.insert(&[0, 1, 2, 3, 4], ());
    trie.insert(&[0, 1, 2, 3, 4, 5, 6], ());
    assert_eq!(
        trie,
        TrieNode {
            value: Some(()),
            prefix: smallvec![],
            branches: mini_vec![TrieNode {
                value: Some(()),
                prefix: smallvec![0, 1, 2],
                branches: mini_vec![TrieNode {
                    value: Some(()),
                    prefix: smallvec![3, 4],
                    branches: mini_vec![TrieNode {
                        value: Some(()),
                        prefix: smallvec![5, 6],
                        branches: mini_vec![]
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
    assert_eq!(48, std::mem::size_of::<TrieNode<()>>());
}
