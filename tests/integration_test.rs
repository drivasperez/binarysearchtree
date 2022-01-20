use binarysearchtree::BinarySearchTree;

#[test]
fn can_make_one() {
    let _: BinarySearchTree<i32> = BinarySearchTree::new();
}

#[test]
fn can_insert_value() {
    let mut tree = BinarySearchTree::new();
    tree.insert(3);
    tree.insert(44);
    tree.insert(5);
}

#[test]
fn can_search_tree() {
    let mut tree = BinarySearchTree::new();
    tree.insert(3);
    tree.insert(44);
    tree.insert(5);

    assert!(tree.contains(&3));
    assert!(tree.contains(&44));
    assert!(tree.contains(&5));

    assert!(!tree.contains(&1));
    assert!(!tree.contains(&1001));
    assert!(!tree.contains(&4));
}

#[test]
fn can_get_refs() {
    let mut tree = BinarySearchTree::new();

    tree.insert(String::from("Hello"));
    tree.insert(String::from("World"));
    tree.insert(String::from("How are you?"));

    let q = tree.get("How are you?").unwrap();

    assert_eq!(q, "How are you?");
}

#[test]
fn can_find_min_and_max() {
    let mut tree = BinarySearchTree::new();

    assert!(tree.min().is_none());
    assert!(tree.max().is_none());

    tree.insert(3);
    tree.insert(44);
    tree.insert(5);

    let &min = tree.min().unwrap();
    let &max = tree.max().unwrap();

    assert_eq!(min, 3);
    assert_eq!(max, 44);
}

#[test]
fn can_delete() {
    let mut tree = BinarySearchTree::new();

    assert!(tree.min().is_none());
    assert!(tree.max().is_none());

    tree.insert(3);
    tree.insert(44);
    tree.insert(5);

    let &min = tree.min().unwrap();
    let &max = tree.max().unwrap();

    assert_eq!(min, 3);
    assert_eq!(max, 44);

    tree.delete(&3);
    tree.delete(&44);
    tree.delete(&5);

    assert!(tree.min().is_none());
    assert!(tree.max().is_none());
}

#[test]
fn drop_check() {
    let mut x = 42;
    let mut tree = BinarySearchTree::new();
    tree.insert(&mut x);

    println!("{}", x);
}

#[test]
fn covariance() {
    let x = String::from("Hello");
    // Should be covariant over T.
    let y = x.as_str();
    let mut tree = BinarySearchTree::new();
    let mut tree2: BinarySearchTree<&'static str> = BinarySearchTree::new();
    tree2.insert("Hi!");
    tree.insert(y);

    tree = tree2;

    assert!(tree.contains("Hi!"));
    assert!(!tree.contains("Hello"));
}
