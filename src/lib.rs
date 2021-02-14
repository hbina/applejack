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
enum InsertType<'a, T> {
    Parent(&'a mut TrieNode<T>, usize, usize),
    Middle(&'a mut TrieNode<T>, usize, usize),
    Inplace(&'a mut TrieNode<T>, usize),
    Append(&'a mut TrieNode<T>, usize),
}

impl Cut {
    pub fn cut(parent_key: &[u8], child_key: &[u8]) -> Cut {
        let idx = parent_key.iter().zip(child_key).position(|(l, r)| l != r);
        if let Some(idx) = idx {
            if idx == 0 {
                Cut::BothBegin
            } else {
                Cut::BothMiddle(idx)
            }
        } else {
            let (plen, clen) = (parent_key.len(), child_key.len());
            match plen.cmp(&clen) {
                std::cmp::Ordering::Less => Cut::Child(plen),
                std::cmp::Ordering::Equal => Cut::BothEnd,
                std::cmp::Ordering::Greater => Cut::Parent(clen),
            }
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct TrieNode<T> {
    pub keys: Vec<Key>,
    pub nodes: Vec<TrieNode<T>>,
    pub values: Vec<Option<T>>,
}

impl<T> Default for TrieNode<T> {
    fn default() -> Self {
        TrieNode {
            keys: vec![],
            nodes: vec![],
            values: vec![],
        }
    }
}

impl<T> TrieNode<T>
where
    T: std::fmt::Debug,
{
    fn find_mut(&mut self, new_key: &[u8], key_idx: usize) -> InsertType<T> {
        println!("inserting> new_key:{:?} key_idx:{}", new_key, key_idx);
        for idx in 0..self.keys.len() {
            let key = &self.keys[idx];
            let child_key = &new_key[key_idx..];
            println!("child_key:{:?}", child_key);
            match Cut::cut(key, child_key) {
                Cut::Parent(p) => return InsertType::Parent(self, idx, p),
                Cut::Child(c) => return self.nodes[idx].find_mut(new_key, key_idx + c),
                Cut::BothBegin => continue,
                Cut::BothMiddle(m) => return InsertType::Middle(self, idx, m),
                Cut::BothEnd => return InsertType::Inplace(self, idx),
            }
        }
        return InsertType::Append(self, key_idx);
    }

    pub fn insert(&mut self, new_key: &[u8], value: T) {
        self.insert_impl(new_key, Some(value))
    }

    pub fn insert_impl(&mut self, new_key: &[u8], value: Option<T>) {
        let node = self.find_mut(new_key, 0);
        println!("found node> node:{:#?}", node);
        match node {
            InsertType::Parent(parent_node, idx, p) => {
                let drained_key = parent_node.keys[idx].drain(p..).collect::<Key>();
                let drained_nodes = parent_node.nodes[idx].nodes.drain(..).collect::<Vec<_>>();
                let drained_value = parent_node.values[idx].take();
                parent_node.nodes[idx].nodes.push(TrieNode {
                    keys: vec![drained_key],
                    nodes: drained_nodes,
                    values: vec![drained_value],
                });
                parent_node.values[idx] = value;
            }
            InsertType::Middle(parent_node, idx, m) => {
                let drained_key = parent_node.keys[idx].drain(m..).collect::<Key>();
                let drained_value = parent_node.values[idx].take();
                parent_node.nodes[idx].insert_impl(&drained_key, drained_value);
                parent_node.nodes[idx].insert_impl(&new_key[m..], value);
            }
            InsertType::Inplace(parent_node, idx) => {
                parent_node.values[idx] = value;
            }
            InsertType::Append(parent_node, k) => {
                parent_node.keys.push(Key::from_slice(&new_key[k..]));
                parent_node.nodes.push(TrieNode::default());
                parent_node.values.push(value);
            }
        }
    }

    pub fn exists(&self, child_key: &[u8]) -> bool {
        self.keys
            .iter()
            .any(|parent_key| match Cut::cut(parent_key, child_key) {
                Cut::Parent(_) => false,
                Cut::Child(idx) => self.nodes.iter().any(|x| x.exists(&parent_key[idx..])),
                Cut::BothBegin => false,
                Cut::BothMiddle(_) => false,
                Cut::BothEnd => true,
            })
    }
}

#[test]
fn small_tests() {
    let mut trie = TrieNode::default();
    println!("trie1:{:#?}", trie);
    trie.insert(&[0, 1, 2], ());
    println!("trie2:{:#?}", trie);
    trie.insert(&[0, 1, 2, 3, 4], ());
    println!("trie3:{:#?}", trie);
}

#[test]
fn general_tests() {
    let mut trie = TrieNode::default();
    println!("trie1:{:#?}", trie);
    trie.insert(&[0, 1, 2], 1);
    println!("trie2:{:#?}", trie);
    trie.insert(&[0, 1, 2, 3, 4], 2);
    println!("trie3:{:#?}", trie);
    trie.insert(&[0, 1, 3], 3);
    println!("trie4:{:#?}", trie);
    trie.insert(&[], 4);
    println!("trie5:{:#?}", trie);
    trie.insert(&[], 5);
    println!("trie6:{:#?}", trie);
    assert_eq!(
        trie,
        TrieNode {
            keys: vec![smallvec![]],
            nodes: vec![TrieNode {
                keys: vec![smallvec![0, 1]],
                nodes: vec![
                    TrieNode {
                        keys: vec![smallvec![2], smallvec![3]],
                        nodes: vec![TrieNode {
                            keys: vec![smallvec![3, 4]],
                            nodes: vec![TrieNode::default()],
                            values: vec![Some(2)],
                        }],
                        values: vec![Some(1), Some(3)]
                    },
                    TrieNode::default()
                ],
                values: vec![None]
            }],
            values: vec![Some(5)]
        }
    );

    assert!(trie.exists(&[0, 1, 2]));
    assert!(trie.exists(&[0, 1, 2, 3, 4]));
    assert!(trie.exists(&[0, 1, 3]));
    assert!(trie.exists(&[]));
    assert!(!trie.exists(&[1, 2, 3]));
}
