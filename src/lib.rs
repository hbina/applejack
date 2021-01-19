#[cfg(test)]
mod tests {
    use crate::TrieNode;

    #[test]
    fn general_tests() {
        let mut trie = TrieNode::new();
        trie.insert(&[0, 1, 2]);
        trie.insert(&[0, 1, 2, 3, 4]);
        trie.insert(&[0, 1, 3]);
        trie.insert(&[]);
        trie.insert(&[]);
        assert_eq!(
            trie,
            TrieNode {
                prefix: vec![],
                branches: vec![TrieNode {
                    prefix: vec![0, 1],
                    branches: vec![
                        TrieNode {
                            prefix: vec![2],
                            branches: vec![TrieNode {
                                prefix: vec![3, 4],
                                branches: vec![]
                            },]
                        },
                        TrieNode {
                            prefix: vec![3],
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
        assert_eq!(
            trie,
            TrieNode {
                prefix: vec![],
                branches: vec![TrieNode {
                    prefix: vec![0, 1, 2],
                    branches: vec![TrieNode {
                        prefix: vec![3, 4],
                        branches: vec![TrieNode {
                            prefix: vec![5, 6],
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
                prefix: vec![],
                branches: vec![
                    TrieNode {
                        prefix: vec![0, 1, 2, 3],
                        branches: vec![]
                    },
                    TrieNode {
                        prefix: vec![4, 5, 6, 7],
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
                prefix: vec![],
                branches: vec![
                    TrieNode {
                        prefix: vec![0, 1, 2, 3],
                        branches: vec![]
                    },
                    TrieNode {
                        prefix: vec![4, 5, 6, 7],
                        branches: vec![]
                    }
                ]
            }
        );
        assert!(trie.get(&[0, 1, 2, 333]));
    }

    #[test]
    fn get_empty_exists() {
        let trie = TrieNode::new();
        assert!(trie.get(&[]));
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
                prefix: vec![],
                branches: vec![TrieNode {
                    prefix: vec![0, 1, 2],
                    branches: vec![TrieNode {
                        prefix: vec![3, 4],
                        branches: vec![TrieNode {
                            prefix: vec![5, 6],
                            branches: vec![]
                        }]
                    }]
                },]
            }
        );
        assert!(!trie.get(&[0, 1, 2, 3]));
        assert!(trie.get(&[0, 1, 2, 3, 4]));
        assert!(!trie.get(&[0, 1, 2, 3, 4, 5]));
        assert!(trie.get(&[0, 1, 2, 3, 4, 5, 6]));
    }
}

#[derive(PartialEq, Debug)]
enum Cut {
    Parent(usize),
    Child(usize),
    BothBegin,
    BothMiddle(usize),
    BothEnd,
}

#[derive(PartialEq, Debug)]
pub struct TrieNode {
    prefix: Vec<u8>,
    branches: Vec<TrieNode>,
}

impl TrieNode {
    pub fn new() -> TrieNode {
        TrieNode {
            prefix: vec![],
            branches: vec![],
        }
    }

    pub fn with_key(key: &[u8]) -> TrieNode {
        TrieNode {
            prefix: Vec::from(key),
            branches: vec![],
        }
    }

    pub fn with_branches(branches: Vec<TrieNode>) -> TrieNode {
        TrieNode {
            prefix: vec![],
            branches,
        }
    }

    pub fn with_key_and_branches(key: &[u8], branches: Vec<TrieNode>) -> TrieNode {
        TrieNode {
            prefix: Vec::from(key),
            branches,
        }
    }


    #[test]
    fn get_empty_exists() {
        let trie = TrieNode::new();
        assert!(trie.get(&[]));
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
                prefix: vec![],
                branches: vec![TrieNode {
                    prefix: vec![0, 1, 2],
                    branches: vec![TrieNode {
                        prefix: vec![3, 4],
                        branches: vec![TrieNode {
                            prefix: vec![5, 6],
                            branches: vec![]
                        }]
                    }]
                },]
            }
        );
        assert!(!trie.get(&[0, 1, 2, 3]));
        assert!(trie.get(&[0, 1, 2, 3, 4]));
        assert!(!trie.get(&[0, 1, 2, 3, 4, 5]));
        assert!(trie.get(&[0, 1, 2, 3, 4, 5, 6]));
    }
}

#[derive(PartialEq, Debug)]
enum Cut {
    Parent(usize),
    Child(usize),
    BothBegin,
    BothMiddle(usize),
    BothEnd,
}

#[derive(PartialEq, Debug)]
pub struct TrieNode {
    prefix: Vec<u8>,
    branches: Vec<TrieNode>,
}

impl TrieNode {
    pub fn new() -> TrieNode {
        TrieNode {
            prefix: vec![],
            branches: vec![],
        }
    }

    pub fn with_key(key: &[u8]) -> TrieNode {
        TrieNode {
            prefix: Vec::from(key),
            branches: vec![],
        }
    }

    pub fn with_branches(branches: Vec<TrieNode>) -> TrieNode {
        TrieNode {
            prefix: vec![],
            branches,
        }
    }

    pub fn with_key_and_branches(key: &[u8], branches: Vec<TrieNode>) -> TrieNode {
        TrieNode {
            prefix: Vec::from(key),
            branches,
        }
    }

    pub fn insert(&mut self, new_key: &[u8]) -> bool {
        let cut = self.cut_key(new_key);
        match cut {
            Cut::Parent(p) => {
                let drained_key = self.prefix.drain(p..).collect();
                let drained_branch = self.branches.drain(..).collect();
                self.branches.push(TrieNode::with_branches(vec![TrieNode {
                    prefix: drained_key,
                    branches: drained_branch,
                }]));
            }
    pub fn insert(&mut self, new_key: &[u8]) -> bool {
        let cut = self.cut_key(new_key);
        match cut {
            Cut::Parent(p) => {
                let drained_key = self.prefix.drain(p..).collect();
                let drained_branch = self.branches.drain(..).collect();
                self.branches.push(TrieNode::with_branches(vec![TrieNode {
                    prefix: drained_key,
                    branches: drained_branch,
                }]));
            }
            Cut::Child(c) => {
                let cut_child = &new_key[c..];
                if !self.branches.iter_mut().any(|x| x.insert(cut_child)) {
                    self.branches.push(TrieNode::with_key(cut_child));
                }
            }
            Cut::BothBegin => return false,
            Cut::BothMiddle(p) => {
                let drained_key = self.prefix.drain(p..).collect::<Vec<_>>();
                let drained_children = self.branches.drain(..).collect();
                self.branches.push(TrieNode::with_key_and_branches(
                    &drained_key,
                    drained_children,
                ));
                self.branches
                    .push(TrieNode::with_key_and_branches(&new_key[p..], vec![]));
            }
            Cut::BothEnd => {}
        }
        true
    }

    pub fn get(&self, key: &[u8]) -> bool {
        match self.cut_key(key) {
            Cut::Parent(_) => false,
            Cut::Child(idx) => self.branches.iter().any(|x| x.get(&key[idx..])),
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
