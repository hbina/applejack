type Key = Vec<u8>;
type Branches<T> = Vec<RaxNode<T>>;

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
    empty: Option<T>,
    branches: Branches<T>,
}

impl<T> Rax<T> {
    pub fn insert(&mut self, key: &[u8], value: T) {
        if key.is_empty() {
            self.empty = Some(value);
        } else if let Some(node) = self.branches.iter_mut().find_map(|n| n.insert_node(key)) {
            node.value = Some(value);
        } else {
            self.branches.push(RaxNode {
                value: Some(value),
                prefix: Key::from(key),
                branches: vec![],
            });
        };
    }

    pub fn exists(&self, key: &[u8]) -> bool {
        (key.is_empty() && self.empty.is_some()) || self.branches.iter().any(|n| n.exists(key))
    }

    pub fn remove(&mut self, key: &[u8]) -> Option<T> {
        if key.is_empty() {
            self.empty.take()
        } else {
            self.branches
                .iter_mut()
                .find_map(|n| n.remove_node(key).map(|s| s.0))
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<&T> {
        if key.is_empty() {
            self.empty.as_ref()
        } else {
            self.branches.iter().find_map(|n| n.get(key))
        }
    }

    pub fn get_mut(&mut self, key: &[u8]) -> Option<&mut T> {
        if key.is_empty() {
            self.empty.as_mut()
        } else {
            self.branches.iter_mut().find_map(|n| n.get_mut(key))
        }
    }
}

// TODO: Add ability to specify the kind of containers to be used for keys and branches?
#[derive(PartialEq, Default, Debug)]
struct RaxNode<T> {
    pub(crate) value: Option<T>,
    pub(crate) prefix: Key,
    pub(crate) branches: Vec<RaxNode<T>>,
}

impl<T> RaxNode<T> {
    fn insert_node(&mut self, key: &[u8]) -> Option<&mut RaxNode<T>> {
        match self.cut_key(key) {
            Cut::Parent(p) => {
                let child_value = self.value.take();
                let child_prefix = self.prefix.drain(p..).collect();
                let child_branches = self.branches.drain(..).collect();
                self.value = None;
                self.branches.push(RaxNode {
                    value: child_value,
                    prefix: child_prefix,
                    branches: child_branches,
                });
                Some(self)
            }
            Cut::Child(c) => {
                let child_key = &key[c..];
                self.branches
                    .iter_mut()
                    .find_map(|n| n.insert_node(child_key))
            }
            Cut::BothBegin => None,
            Cut::BothMiddle(m) => {
                let left_value = None;
                let left_prefix = Key::from(&key[m..]);
                let left_branches = vec![];
                let left_node = RaxNode {
                    value: left_value,
                    prefix: left_prefix,
                    branches: left_branches,
                };

                let right_value = self.value.take();
                let right_prefix = self.prefix.drain(m..).collect();
                let right_branches = self.branches.drain(..).collect();
                let right_node = RaxNode {
                    value: right_value,
                    prefix: right_prefix,
                    branches: right_branches,
                };

                self.branches.reserve(2);
                self.branches.push(left_node);
                self.branches.push(right_node);

                self.branches.get_mut(0)
            }
            Cut::BothEnd => Some(self),
        }
    }

    fn remove_node(&mut self, key: &[u8]) -> Option<(T, bool)> {
        match self.cut_key(key) {
            Cut::Parent(_) => None,
            Cut::Child(c) => {
                let child_node = self.branches.iter_mut().enumerate().find_map(|(idx, n)| {
                    n.remove_node(&key[c..])
                        .map(|(v, should_delete)| (idx, v, should_delete))
                });
                if let Some((idx, _, should_delete)) = child_node.as_ref() {
                    if *should_delete {
                        self.branches.swap_remove(*idx);
                    }
                }
                child_node.map(|s| (s.1, false))
            }
            Cut::BothBegin => None,
            Cut::BothMiddle(_) => None,
            Cut::BothEnd => self.value.take().map(|s| (s, self.branches.is_empty())),
        }
    }

    fn exists(&self, key: &[u8]) -> bool {
        self.get(key).is_some()
    }

    fn get(&self, key: &[u8]) -> Option<&T> {
        self.find_node(key).map(|n| n.value.as_ref()).flatten()
    }

    fn get_mut(&mut self, key: &[u8]) -> Option<&mut T> {
        self.find_node_mut(key).map(|n| n.value.as_mut()).flatten()
    }
}

impl<T> RaxNode<T> {
    fn find_node<'this, 'key>(&'this self, key: &'key [u8]) -> Option<&RaxNode<T>> {
        match self.cut_key(key) {
            Cut::Parent(_) => None,
            Cut::Child(c) => {
                let child_key = &key[c..];
                self.branches.iter().find_map(|s| s.find_node(child_key))
            }
            Cut::BothBegin => None,
            Cut::BothMiddle(_) => None,
            Cut::BothEnd => Some(self),
        }
    }

    fn find_node_mut<'this, 'key>(&'this mut self, key: &'key [u8]) -> Option<&mut RaxNode<T>> {
        match self.cut_key(key) {
            Cut::Parent(_) => None,
            Cut::Child(c) => {
                let child_key = &key[c..];
                self.branches
                    .iter_mut()
                    .find_map(|s| s.find_node_mut(child_key))
            }
            Cut::BothBegin => None,
            Cut::BothMiddle(_) => None,
            Cut::BothEnd => Some(self),
        }
    }

    fn cut_key(&self, key: &'_ [u8]) -> Cut {
        let idx = self.prefix.iter().zip(key).position(|(l, r)| l != r);
        if let Some(idx) = idx {
            if idx == 0 {
                Cut::BothBegin
            } else {
                Cut::BothMiddle(idx)
            }
        } else {
            let (plen, clen) = (self.prefix.len(), key.len());
            match plen.cmp(&clen) {
                std::cmp::Ordering::Less => Cut::Child(plen),
                std::cmp::Ordering::Equal => Cut::BothEnd,
                std::cmp::Ordering::Greater => Cut::Parent(clen),
            }
        }
    }
}
