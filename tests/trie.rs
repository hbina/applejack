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
    println!("1:{:#?}", root);
}

#[test]
pub fn basic_insert_and_remove_stuff() {
    let a = b"hello alice";
    let b = b"hello kyle";
    let mut root = Rax::default();
    println!("0:{:#?}", root);
    root.insert(a, 10);
    println!("1:{:#?}", root);
    root.insert(b, 20);
    println!("2:{:#?}", root);
    assert!(root.exists(a));
    assert!(root.exists(b));
    println!("3:{:#?}", root);
    assert_eq!(root.remove(a), Some(10));
    println!("4:{:#?}", root);
    assert!(!root.exists(a));
    assert_eq!(root.remove(b), Some(20));
    println!("5:{:#?}", root);
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
