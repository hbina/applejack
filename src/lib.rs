use minivec::{mini_vec, MiniVec};
use smallvec::SmallVec;

pub type Key = SmallVec<[u8; 16]>;

#[derive(PartialEq, Debug)]
enum Cut {
    Parent(usize),
    Child(usize),
    BothBegin,
    BothMiddle(usize),
    BothEnd,
}

#[derive(Default, Debug)]
pub struct Rax<T> {
    node: RaxNode<T>,
}

impl<T> Rax<T> {
    pub fn insert(&mut self, new_key: &[u8], value: T) {
        self.node.insert_impl(new_key, value);
    }

    pub fn exists(&self, key: &[u8]) -> bool {
        self.node.exists(key)
    }

    pub fn remove(&mut self, key: &[u8]) -> Option<T> {
        self.node.remove(key)
    }

    pub fn get(&self, key: &[u8]) -> Option<&T> {
        self.node.get(key)
    }
}

// TODO: Add ability to specify the kind of containers to be used for keys and branches?
#[derive(PartialEq, Default, Debug)]
struct RaxNode<T> {
    pub(crate) value: Option<T>,
    pub(crate) prefix: Key,
    pub(crate) branches: MiniVec<RaxNode<T>>,
}

impl<T> RaxNode<T> {
    fn exists(&self, key: &[u8]) -> bool {
        match self.cut_key(key) {
            Cut::Parent(_) => false,
            Cut::Child(idx) => self.branches.iter().any(|x| x.exists(&key[idx..])),
            Cut::BothBegin => false,
            Cut::BothMiddle(_) => false,
            Cut::BothEnd => self.value.is_some(),
        }
    }

    fn remove(&mut self, key: &[u8]) -> Option<T> {
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

    fn get(&self, key: &[u8]) -> Option<&T> {
        match self.cut_key(key) {
            Cut::Parent(_) => None,
            Cut::Child(idx) => self.branches.iter().find_map(|x| x.get_impl(&key[idx..])),
            Cut::BothBegin => None,
            Cut::BothMiddle(_) => None,
            Cut::BothEnd => self.value.as_ref(),
        }
    }
}

impl<T> RaxNode<T> {
    pub fn insert_impl(&mut self, new_key: &[u8], value: T) {
        let value = &mut Some(value);
        let inserted = self.insert_impl_inner(new_key, value);
        if !inserted {
            let drained_value = self.value.take();
            let drained_key = self.prefix.drain(..).collect::<Key>();
            let drained_children = self.branches.drain(..).collect();
            self.branches.push(RaxNode {
                value: drained_value,
                prefix: drained_key,
                branches: drained_children,
            });
            self.branches.push(RaxNode {
                value: value.take(),
                prefix: Key::from(new_key),
                branches: mini_vec![],
            });
        }
    }

    fn insert_impl_inner(&mut self, new_key: &[u8], value: &mut Option<T>) -> bool {
        match self.cut_key(new_key) {
            Cut::Parent(p) => {
                let drained_value = self.value.take();
                self.value = value.take();
                let drained_key = self.prefix.drain(p..).collect::<Key>();
                let drained_branch = self.branches.drain(..).collect();
                self.branches.push(RaxNode {
                    value: drained_value,
                    prefix: drained_key,
                    branches: drained_branch,
                });
                true
            }
            Cut::Child(c) => {
                let cut_child = &new_key[c..];
                if self.prefix.is_empty() && self.branches.is_empty() {
                    self.prefix = cut_child.into();
                    self.value = value.take();
                } else {
                    let found = self
                        .branches
                        .iter_mut()
                        .any(|x| x.insert_impl_inner(cut_child, value));
                    if !found {
                        self.branches.push(RaxNode {
                            value: value.take(),
                            prefix: Key::from(cut_child),
                            branches: mini_vec![],
                        });
                    }
                }
                true
            }
            Cut::BothBegin => false,
            Cut::BothMiddle(p) => {
                let drained_value = self.value.take();
                let drained_key = self.prefix.drain(p..).collect::<Key>();
                let drained_children = self.branches.drain(..).collect();
                self.branches.push(RaxNode {
                    value: drained_value,
                    prefix: drained_key,
                    branches: drained_children,
                });
                self.branches.push(RaxNode {
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

    // TODO: Should reimplement this into 2 parts: function that gets the mut& requested node and then mutating said node.
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

    fn get_impl(&self, key: &[u8]) -> Option<&T> {
        match self.cut_key(key) {
            Cut::Parent(_) => None,
            Cut::Child(key_idx) => self
                .branches
                .iter()
                .find_map(|x| x.get_impl(&key[key_idx..]).map(|value| (value))),
            Cut::BothBegin => None,
            Cut::BothMiddle(_) => None,
            Cut::BothEnd => self.value.as_ref(),
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
            let (plen, clen) = (self.prefix.len(), child_key.len());
            match plen.cmp(&clen) {
                std::cmp::Ordering::Less => Cut::Child(plen),
                std::cmp::Ordering::Equal => Cut::BothEnd,
                std::cmp::Ordering::Greater => Cut::Parent(clen),
            }
        }
    }
}

#[test]
fn general_tests() {
    let mut trie = RaxNode::default();
    trie.insert_impl(&[0, 1, 2], ());
    trie.insert_impl(&[0, 1, 2, 3, 4], ());
    trie.insert_impl(&[0, 1, 3], ());
    trie.insert_impl(&[], ());
    trie.insert_impl(&[], ());
    assert!(trie.exists(&[0, 1, 2]));
    assert!(trie.exists(&[0, 1, 2, 3, 4]));
    assert!(trie.exists(&[0, 1, 3]));
    assert!(trie.exists(&[]));
    assert!(!trie.exists(&[1, 2, 3]));
    assert_eq!(
        trie,
        RaxNode {
            value: Some(()),
            prefix: smallvec::smallvec![],
            branches: mini_vec![RaxNode {
                value: None,
                prefix: smallvec::smallvec![0, 1],
                branches: mini_vec![
                    RaxNode {
                        value: Some(()),
                        prefix: smallvec::smallvec![2],
                        branches: mini_vec![RaxNode {
                            value: Some(()),
                            prefix: smallvec::smallvec![3, 4],
                            branches: mini_vec![]
                        },]
                    },
                    RaxNode {
                        value: Some(()),
                        prefix: smallvec::smallvec![3],
                        branches: mini_vec![]
                    }
                ]
            },]
        }
    )
}

#[test]
fn insert_empty() {
    let mut trie = RaxNode::default();
    trie.insert_impl(&[0, 1, 2], ());
    trie.insert_impl(&[], ());
    trie.insert_impl(&[0, 1, 2, 3, 4], ());
    trie.insert_impl(&[0, 1, 2, 3, 4, 5, 6], ());
    assert!(trie.exists(&[0, 1, 2]));
    assert!(trie.exists(&[]));
    assert!(!trie.exists(&[0, 1, 2, 3]));
    assert!(!trie.exists(&[0, 1]));
    assert_eq!(
        trie,
        RaxNode {
            value: Some(()),
            prefix: smallvec::smallvec![],
            branches: mini_vec![RaxNode {
                value: Some(()),
                prefix: smallvec::smallvec![0, 1, 2],
                branches: mini_vec![RaxNode {
                    value: Some(()),
                    prefix: smallvec::smallvec![3, 4],
                    branches: mini_vec![RaxNode {
                        value: Some(()),
                        prefix: smallvec::smallvec![5, 6],
                        branches: mini_vec![]
                    }]
                }]
            },]
        }
    )
}

#[test]
fn insert_very_different_strings() {
    let mut trie = RaxNode::default();
    trie.insert_impl(&[0, 1, 2, 3], ());
    trie.insert_impl(&[4, 5, 6, 7], ());
    assert_eq!(
        trie,
        RaxNode {
            value: None,
            prefix: smallvec::smallvec![],
            branches: mini_vec![
                RaxNode {
                    value: Some(()),
                    prefix: smallvec::smallvec![0, 1, 2, 3],
                    branches: mini_vec![]
                },
                RaxNode {
                    value: Some(()),
                    prefix: smallvec::smallvec![4, 5, 6, 7],
                    branches: mini_vec![]
                }
            ]
        }
    )
}

#[test]
fn get_something_that_exist() {
    let mut trie = RaxNode::default();
    trie.insert_impl(&[0, 1, 2, 3], ());
    println!("1:\n{:#?}", trie);
    trie.insert_impl(&[4, 5, 6, 7], ());
    println!("2:\n{:#?}", trie);
    assert_eq!(
        trie,
        RaxNode {
            value: None,
            prefix: smallvec::smallvec![],
            branches: mini_vec![
                RaxNode {
                    value: Some(()),
                    prefix: smallvec::smallvec![0, 1, 2, 3],
                    branches: mini_vec![]
                },
                RaxNode {
                    value: Some(()),
                    prefix: smallvec::smallvec![4, 5, 6, 7],
                    branches: mini_vec![]
                }
            ]
        }
    );
    assert!(trie.exists(&[0, 1, 2, 3]));
}

#[test]
fn initialize_with_something_big() {
    let mut trie = RaxNode::default();
    trie.insert_impl(&[0, 1, 2, 3], ());
    trie.insert_impl(&[0, 1, 2, 3, 4], ());
    assert_eq!(
        trie,
        RaxNode {
            value: Some(()),
            prefix: smallvec::smallvec![0, 1, 2, 3],
            branches: mini_vec![RaxNode {
                value: Some(()),
                prefix: smallvec::smallvec![4],
                branches: mini_vec![]
            },]
        }
    );
    assert!(trie.exists(&[0, 1, 2, 3]));
}

#[test]
fn get_empty_exists() {
    let trie = RaxNode::<()>::default();
    assert!(!trie.exists(&[]));
}

#[test]
fn get_nested_exists() {
    let mut trie = RaxNode::default();
    trie.insert_impl(&[0, 1, 2], ());
    trie.insert_impl(&[], ());
    trie.insert_impl(&[0, 1, 2, 3, 4], ());
    trie.insert_impl(&[0, 1, 2, 3, 4, 5, 6], ());
    assert_eq!(
        trie,
        RaxNode {
            value: Some(()),
            prefix: smallvec::smallvec![],
            branches: mini_vec![RaxNode {
                value: Some(()),
                prefix: smallvec::smallvec![0, 1, 2],
                branches: mini_vec![RaxNode {
                    value: Some(()),
                    prefix: smallvec::smallvec![3, 4],
                    branches: mini_vec![RaxNode {
                        value: Some(()),
                        prefix: smallvec::smallvec![5, 6],
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
    assert_eq!(48, std::mem::size_of::<RaxNode<()>>());
    assert_eq!(8, std::mem::size_of::<MiniVec<RaxNode<()>>>());
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
    let mut trie = RaxNode::default();
    for x in input.iter() {
        trie.insert_impl(&x, ());
    }
    assert!(input.iter().all(|x| { trie.exists(x) }));
}

#[test]
fn test_fuzzy_input_1_minimized() {
    let input = [mini_vec![230, 45], mini_vec![230, 85]];
    let mut trie = RaxNode::default();
    for x in input.iter() {
        trie.insert_impl(&x, ());
    }
    assert!(input.iter().all(|x| { trie.exists(x) }));
}

#[test]
fn test_multiple_insert_and_remove() {
    let key1 = [230, 45];
    let key2 = [230, 85];
    let key3 = [230, 100];
    let mut trie = RaxNode::default();
    assert!(!trie.exists(&key1));
    assert!(!trie.exists(&key2));
    trie.insert_impl(&key1, ());
    assert!(trie.exists(&key1));
    assert!(!trie.exists(&key2));
    trie.insert_impl(&key2, ());
    assert!(trie.exists(&key1));
    assert!(trie.exists(&key2));
    trie.remove(&key1);
    assert!(!trie.exists(&key1));
    assert!(trie.exists(&key2));
    trie.remove(&key2);
    assert!(!trie.exists(&key1));
    assert!(!trie.exists(&key2));
    trie.insert_impl(&key3, ());
}

#[test]
fn test_insert_and_remove() {
    let key1 = [230, 45];
    let mut trie = RaxNode::default();
    assert!(!trie.exists(&key1));
    trie.insert_impl(&key1, ());
    assert!(trie.exists(&key1));
    trie.remove(&key1);
    assert!(!trie.exists(&key1));
}

#[test]
fn test_insert_and_remove_empty_bytes() {
    let key1 = [];
    let mut trie = RaxNode::default();
    assert!(!trie.exists(&key1));
    trie.insert_impl(&key1, ());
    assert!(trie.exists(&key1));
    trie.remove(&key1);
    assert!(!trie.exists(&key1));
}

#[test]
fn test_size_of_container() {
    let key1 = [];
    let mut trie = RaxNode::default();
    assert_eq!(std::mem::size_of_val(&trie), 48);
    assert!(!trie.exists(&key1));
    trie.insert_impl(&key1, ());
    assert!(trie.exists(&key1));
    trie.remove(&key1);
    assert!(!trie.exists(&key1));
}
