use applejack::Rax;

#[test]
pub fn basic_test() {
    let mut root = Rax::<()>::default();
    assert!(!root.exists(&[]));
    root.insert(&[], ());
    assert!(root.exists(&[]));
}

#[test]
pub fn basic_insert_stuff() {
    let a = b"hello alice";
    let b = b"hello kyle";
    let mut root = Rax::default();
    root.insert(a, ());
    root.insert(b, ());
    assert!(root.exists(a));
    assert!(root.exists(b));
    assert!(!root.exists(b"h"));
    assert!(!root.exists(b"hello"));
    assert!(!root.exists(b"alice"));
    assert!(!root.exists(&[]));
}

#[test]
pub fn insert_1_element() {
    let a = b"hello alice";
    let mut root = Rax::default();
    root.insert(a, 10);
    assert!(root.exists(a));
}

#[test]
pub fn basic_insert_and_remove_stuff() {
    let a = b"hello alice";
    let b = b"hello kyle";
    let mut root = Rax::default();
    root.insert(a, 10);
    root.insert(b, 20);
    assert!(root.exists(a));
    assert!(root.exists(b));
    assert_eq!(root.remove(a), Some(10));
    assert!(!root.exists(a));
    assert_eq!(root.remove(b), Some(20));
    assert!(!root.exists(b));
}

#[test]
pub fn basic_insert_and_get_stuff() {
    let a = b"hello alice";
    let b = b"hello kyle";
    let mut root = Rax::default();
    root.insert(a, 10);
    root.insert(b, 20);
    assert_eq!(root.get(a), Some(&10));
    assert_eq!(root.get(b), Some(&20));
    assert_eq!(root.get(&[]), None);
    assert_eq!(root.get(b"hello"), None);
    assert_eq!(root.get(b"alice"), None);
}

#[test]
pub fn insert_and_remove_random_values() {
    let a = b"hello alice";
    let b = b"hello kyle";
    let mut root = Rax::default();
    root.insert(a, 10);
    root.insert(b, 20);
    assert_eq!(root.get(a), Some(&10));
    assert_eq!(root.get(b), Some(&20));
    assert_eq!(root.get(&[]), None);
    assert_eq!(root.get(b"hello"), None);
    assert_eq!(root.get(b"alice"), None);
}

#[test]
pub fn test_remove_remove_remove() {
    let a = b"hello kye";
    let mut root = Rax::default();
    root.remove(a);
    assert!(!root.exists(a));
    root.insert(a, ());
    assert!(root.exists(a));
    root.remove(a);
    root.remove(a);
    assert!(!root.exists(a));
}

#[test]
pub fn test_get_empty() {
    let root = Rax::<()>::default();
    assert!(!root.exists(&[]));
}

#[test]
pub fn test_insert_empty() {
    let mut root = Rax::default();
    root.insert(&[], ());
    root.insert(&[], ());
    root.insert(b"hello", ());
    root.remove(b"hello");
    assert!(root.exists(&[]));
}

#[derive(Debug)]
enum Operation {
    Insert(&'static [u8], u8),
    Remove(&'static [u8]),
}

/// See https://github.com/hbina/applejack/pull/9
#[test]
pub fn test_only_delete_node_if_no_branches() {
    let data = [
        Operation::Insert(&[101, 212, 101, 101, 40, 83, 101, 101], 101),
        Operation::Insert(&[101, 101, 101, 83, 83], 0),
        Operation::Insert(&[101, 101], 233),
        Operation::Insert(&[101, 101, 101, 101, 101], 212),
        Operation::Remove(&[101, 101]),
        Operation::Remove(&[101, 101, 101, 101, 101]),
    ];
    let mut table = std::collections::HashMap::new();
    let mut rax = Rax::default();
    for operation in &data {
        match operation {
            Operation::Insert(key, value) => {
                rax.insert(key, *value);
                table.insert(key, *value);
            }
            Operation::Remove(key) => {
                assert_eq!(rax.remove(key), table.remove(key));
            }
        };
    }
}
