#[cfg(test)]
mod tests {
    use crate::TrieNode;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn tryyy() {
        let mut trie = TrieNode::new();
        trie.insert(&[0, 1, 2]);
        println!("1. trie:{:#?}", trie);
        trie.insert(&[0, 1, 2, 3, 4]);
        println!("2. trie:{:#?}", trie);
        trie.insert(&[0, 1, 3]);
        println!("3. trie:{:#?}", trie);
        trie.insert(&[]);
        println!("4. trie:{:#?}", trie);
        trie.insert(&[]);
        println!("5. trie:{:#?}", trie);
    }
}

#[derive(PartialEq)]
enum Cut {
    Parent(usize),
    Child(usize),
    BothBegin,
    BothMiddle(usize),
    BothEnd,
}

#[derive(Debug)]
pub struct TrieBranch {
    prefix: Vec<u8>,
    nodes: Vec<TrieNode>,
}

impl TrieBranch {
    pub fn new(key: &[u8]) -> TrieBranch {
        TrieBranch {
            prefix: Vec::from(key),
            nodes: vec![],
        }
    }

    pub fn with_nodes(prefix: Vec<u8>, nodes: Vec<TrieNode>) -> TrieBranch {
        TrieBranch { prefix, nodes }
    }
}

#[derive(Debug)]
pub struct TrieNode {
    branches: Vec<TrieBranch>,
}

impl TrieNode {
    pub fn new() -> TrieNode {
        TrieNode { branches: vec![] }
    }

    pub fn from_branches(branches: Vec<TrieBranch>) -> TrieNode {
        TrieNode { branches }
    }

    pub fn insert(&mut self, new_key: &[u8]) {
        // Use some other enum to make this unrepresentable
        let cut = self.branches.iter().enumerate().find_map(|(i, t)| {
            if let (Some(l), Some(r)) = (t.prefix.iter().next(), new_key.iter().next()) {
                if l == r {
                    Some(i)
                } else {
                    None
                }
            } else {
                None
            }
        });
        if let Some(i) = cut {
            let TrieBranch { prefix, nodes } = &mut self.branches[i];
            match cut_key(prefix, new_key) {
                Cut::Parent(p) => {
                    let drained_key = prefix.drain(p..).collect();
                    let drained_branch = nodes.drain(..).collect();
                    nodes.push(TrieNode::from_branches(vec![TrieBranch::with_nodes(
                        drained_key,
                        drained_branch,
                    )]));
                }
                Cut::Child(c) => nodes.push(TrieNode::from_branches(vec![TrieBranch::with_nodes(
                    Vec::from(&new_key[c + 1..]),
                    vec![],
                )])),
                Cut::BothBegin => {
                    unreachable!()
                }
                Cut::BothMiddle(p) => {
                    let drained_key = prefix.drain(p..).collect();
                    let drained_children = nodes.drain(..).collect();
                    nodes.push(TrieNode::from_branches(vec![TrieBranch::with_nodes(
                        drained_key,
                        drained_children,
                    )]));
                    nodes.push(TrieNode::from_branches(vec![TrieBranch::with_nodes(
                        Vec::from(&new_key[p..]),
                        vec![],
                    )]));
                }
                Cut::BothEnd => {
                    // Just write in-place
                }
            }
        } else {
            self.branches
                .push(TrieBranch::with_nodes(Vec::from(new_key), vec![]));
        }
    }
}

fn cut_key<'a, 'b>(prefix_key: &'a [u8], child_key: &'b [u8]) -> Cut {
    let idx = prefix_key.iter().zip(child_key).position(|(l, r)| l != r);
    if let Some(idx) = idx {
        // SAFETY: The substraction below is safe because idx is guaranteed to be smaller than both lengths
        if idx == 0 {
            Cut::BothBegin
        } else {
            Cut::BothMiddle(idx)
        }
    } else {
        let (llen, clen) = (prefix_key.len(), child_key.len());
        match llen.cmp(&clen) {
            std::cmp::Ordering::Less => Cut::Child(clen - llen),
            std::cmp::Ordering::Equal => Cut::BothEnd,
            std::cmp::Ordering::Greater => Cut::Parent(llen - clen),
        }
    }
}
