use minivec::{mini_vec, MiniVec};
use smallvec::{smallvec, SmallVec};

type Key = SmallVec<[u8; 16]>;

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

    pub fn exists(&self, key: &[u8]) -> bool {
        match self.cut_key(key) {
            Cut::Parent(_) => false,
            Cut::Child(idx) => self.branches.iter().any(|x| x.exists(&key[idx..])),
            Cut::BothBegin => false,
            Cut::BothMiddle(_) => false,
            Cut::BothEnd => self.value.is_some(),
        }
    }

    pub fn remove(&mut self, key: &[u8]) -> Option<T> {
        match self.cut_key(key) {
            Cut::Parent(_) => None,
            Cut::Child(idx) => self
                .branches
                .iter_mut()
                .find_map(|x| x.remove_impl(&key[idx..]).map(|v| v.1)),
            Cut::BothBegin => None,
            Cut::BothMiddle(_) => None,
            Cut::BothEnd => {
                self.prefix.clear();
                return self.value.take();
            }
        }
    }
}

impl<T> TrieNode<T> {
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
            Cut::BothBegin => false,
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

    fn remove_impl(&mut self, key: &[u8]) -> Option<(bool, T)> {
        match self.cut_key(key) {
            Cut::Parent(_) => None,
            Cut::Child(key_idx) => {
                let result = self
                    .branches
                    .iter_mut()
                    .enumerate()
                    .find_map(|(child_idx, x)| {
                        x.remove_impl(&key[key_idx..])
                            .map(|(should_delete, value)| (child_idx, should_delete, value))
                    });

                match result {
                    Some((child_idx, true, _)) => {
                        let result = self.branches.remove(child_idx);
                        return result.value.map(|v| (false, v));
                    }
                    Some((_, false, value)) => return Some((false, value)),
                    _ => return None,
                }
            }
            Cut::BothBegin => None,
            Cut::BothMiddle(_) => None,
            Cut::BothEnd => {
                self.prefix.clear();
                return self.value.take().map(|v| (true, v));
            }
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
    assert!(!trie.exists(&[]));
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
    assert_eq!(8, std::mem::size_of::<MiniVec<TrieNode<()>>>());
    assert_eq!(32, std::mem::size_of::<SmallVec<[u8; 0]>>());
    assert_eq!(32, std::mem::size_of::<Key>());
    assert_eq!(48, std::mem::size_of::<(Vec<()>, Vec<()>)>());
}

#[test]
fn test_fuzzy_input_1() {
    let input = [
        mini_vec![114],
        mini_vec![114],
        mini_vec![109],
        mini_vec![244],
        mini_vec![40],
        mini_vec![66],
        mini_vec![2],
        mini_vec![0],
        mini_vec![38],
        mini_vec![137],
        mini_vec![3],
        mini_vec![31],
        mini_vec![222],
        mini_vec![64],
        mini_vec![61],
        mini_vec![46],
        mini_vec![33],
        mini_vec![245],
        mini_vec![128],
        mini_vec![42],
        mini_vec![243],
        mini_vec![188],
        mini_vec![165],
        mini_vec![224],
        mini_vec![82],
        mini_vec![37],
        mini_vec![232],
        mini_vec![73],
        mini_vec![196],
        mini_vec![240],
        mini_vec![168],
        mini_vec![131],
        mini_vec![36],
        mini_vec![59],
        mini_vec![25],
        mini_vec![129],
        mini_vec![17],
        mini_vec![1],
        mini_vec![239],
        mini_vec![105],
        mini_vec![221],
        mini_vec![39],
        mini_vec![47],
        mini_vec![44],
        mini_vec![152],
        mini_vec![250],
        mini_vec![149],
        mini_vec![14],
        mini_vec![205],
        mini_vec![223],
        mini_vec![255],
        mini_vec![72],
        mini_vec![93, 254],
        mini_vec![31, 25],
        mini_vec![31, 25],
        mini_vec![6, 0],
        mini_vec![143, 222],
        mini_vec![49, 140],
        mini_vec![0, 1],
        mini_vec![0, 1],
        mini_vec![2, 27],
        mini_vec![2, 27],
        mini_vec![9, 25],
        mini_vec![255, 223],
        mini_vec![255, 223],
        mini_vec![37, 25],
        mini_vec![37, 25],
        mini_vec![230, 45],
        mini_vec![0, 25],
        mini_vec![42, 96],
        mini_vec![42, 96],
        mini_vec![1, 0],
        mini_vec![1, 0],
        mini_vec![0, 20],
        mini_vec![2, 45],
        mini_vec![255, 255],
        mini_vec![244, 244],
        mini_vec![244, 244],
        mini_vec![45, 36],
        mini_vec![135, 25],
        mini_vec![28, 0],
        // The next line will crash
        mini_vec![230, 85],
    ];
    let mut trie = TrieNode::default();
    for x in input.iter() {
        trie.insert(&x, ());
    }
    assert!(input.iter().all(|x| { trie.exists(x) }));
}

#[test]
fn test_fuzzy_input_1_minimized() {
    let input = [mini_vec![230, 45], mini_vec![230, 85]];
    let mut trie = TrieNode::default();
    for x in input.iter() {
        trie.insert(&x, ());
    }
    assert!(input.iter().all(|x| { trie.exists(x) }));
}

#[test]
fn test_multiple_insert_and_remove() {
    let key1 = [230, 45];
    let key2 = [230, 85];
    let key3 = [230, 100];
    let mut trie = TrieNode::default();
    assert!(!trie.exists(&key1));
    assert!(!trie.exists(&key2));
    trie.insert(&key1, ());
    assert!(trie.exists(&key1));
    assert!(!trie.exists(&key2));
    trie.insert(&key2, ());
    assert!(trie.exists(&key1));
    assert!(trie.exists(&key2));
    trie.remove(&key1);
    assert!(!trie.exists(&key1));
    assert!(trie.exists(&key2));
    trie.remove(&key2);
    assert!(!trie.exists(&key1));
    assert!(!trie.exists(&key2));
    trie.insert(&key3, ());
}

#[test]
fn test_insert_and_remove() {
    let key1 = [230, 45];
    let mut trie = TrieNode::default();
    assert!(!trie.exists(&key1));
    trie.insert(&key1, ());
    assert!(trie.exists(&key1));
    trie.remove(&key1);
    assert!(!trie.exists(&key1));
}

#[test]
fn test_insert_and_remove_empty_bytes() {
    let key1 = [];
    let mut trie = TrieNode::default();
    assert!(!trie.exists(&key1));
    trie.insert(&key1, ());
    assert!(trie.exists(&key1));
    trie.remove(&key1);
    assert!(!trie.exists(&key1));
}
