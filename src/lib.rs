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

impl<T> TrieNode<T>
where
    T: std::fmt::Debug,
{
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
            Cut::BothBegin => false,
            Cut::BothMiddle(p) => {
                let drained_value = dbg!(self.value.take());
                let drained_key = dbg!(self.prefix.drain(p..).collect::<Key>());
                let drained_children = dbg!(self.branches.drain(..).collect());
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

#[test]
fn test_fuzzy_input_1() {
    let input = [
        vec![114],
        vec![114],
        vec![109],
        vec![244],
        vec![40],
        vec![66],
        vec![2],
        vec![0],
        vec![38],
        vec![137],
        vec![3],
        vec![31],
        vec![222],
        vec![64],
        vec![61],
        vec![46],
        vec![33],
        vec![245],
        vec![128],
        vec![42],
        vec![243],
        vec![188],
        vec![165],
        vec![224],
        vec![82],
        vec![37],
        vec![232],
        vec![73],
        vec![196],
        vec![240],
        vec![168],
        vec![131],
        vec![36],
        vec![59],
        vec![25],
        vec![129],
        vec![17],
        vec![1],
        vec![239],
        vec![105],
        vec![221],
        vec![39],
        vec![47],
        vec![44],
        vec![152],
        vec![250],
        vec![149],
        vec![14],
        vec![205],
        vec![223],
        vec![255],
        vec![72],
        vec![93, 254],
        vec![31, 25],
        vec![31, 25],
        vec![6, 0],
        vec![143, 222],
        vec![49, 140],
        vec![0, 1],
        vec![0, 1],
        vec![2, 27],
        vec![2, 27],
        vec![9, 25],
        vec![255, 223],
        vec![255, 223],
        vec![37, 25],
        vec![37, 25],
        vec![230, 45],
        vec![0, 25],
        vec![42, 96],
        vec![42, 96],
        vec![1, 0],
        vec![1, 0],
        vec![0, 20],
        vec![2, 45],
        vec![255, 255],
        vec![244, 244],
        vec![244, 244],
        vec![45, 36],
        vec![135, 25],
        vec![28, 0],
        // The next line will crash
        vec![230, 85],
    ];
    let mut trie = TrieNode::default();
    for x in input.iter() {
        trie.insert(&x, ());
    }
    println!("trie:\n{:#?}", trie);
    assert!(input.iter().all(|x| { trie.exists(x) }));
}

#[test]
fn test_fuzzy_input_1_minimized() {
    let input = [vec![230, 45], vec![230, 85]];
    let mut trie = TrieNode::default();
    for x in input.iter() {
        trie.insert(&x, ());
    }
    assert!(input.iter().all(|x| { trie.exists(x) }));
}
