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
pub struct TrieNode {
    pub prefix: Key,
    pub branches: Vec<TrieNode>,
}

impl TrieNode {
    pub fn new() -> TrieNode {
        TrieNode {
            prefix: smallvec![],
            branches: vec![],
        }
    }

    pub fn with_key(key: &[u8]) -> TrieNode {
        TrieNode {
            prefix: Key::from(key),
            branches: vec![],
        }
    }

    pub fn with_branches(branches: Vec<TrieNode>) -> TrieNode {
        TrieNode {
            prefix: smallvec![],
            branches,
        }
    }

    pub fn with_key_and_branches(key: &[u8], branches: Vec<TrieNode>) -> TrieNode {
        TrieNode {
            prefix: Key::from(key),
            branches,
        }
    }

    pub fn insert(&mut self, new_key: &[u8]) {
        self.insert_impl(new_key);
    }

    fn insert_impl(&mut self, new_key: &[u8]) -> bool {
        let cut = self.cut_key(new_key);
        match cut {
            Cut::Parent(p) => {
                let drained_key = self.prefix.drain(p..).collect::<Key>();
                let drained_branch = self.branches.drain(..).collect();
                self.branches.push(TrieNode {
                    prefix: drained_key,
                    branches: drained_branch,
                });
                true
            }
            Cut::Child(c) => {
                let cut_child = &new_key[c..];
                if !self
                    .branches
                    .iter_mut()
                    .any(move |x| x.insert_impl(cut_child))
                {
                    self.branches.push(TrieNode::with_key(cut_child));
                }
                true
            }
            Cut::BothBegin => return false,
            Cut::BothMiddle(p) => {
                let drained_key = self.prefix.drain(p..).collect::<Key>();
                let drained_children = self.branches.drain(..).collect();
                self.branches.push(TrieNode {
                    prefix: drained_key,
                    branches: drained_children,
                });
                self.branches.push(TrieNode {
                    prefix: Key::from(&new_key[p..]),
                    branches: vec![],
                });
                true
            }
            Cut::BothEnd => true,
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
