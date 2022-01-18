use std::{borrow::Borrow, marker::PhantomData};

pub struct BinarySearchTree<'a, T> {
    root: *mut Node<'a, T>,
    _phantom: PhantomData<&'a T>,
}

struct Node<'a, T> {
    item: T,
    parent: *mut Node<'a, T>,
    left: *mut Node<'a, T>,
    right: *mut Node<'a, T>,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T> Node<'a, T> {
    pub fn new(item: T) -> Self {
        Self {
            item,
            parent: std::ptr::null_mut(),
            left: std::ptr::null_mut(),
            right: std::ptr::null_mut(),
            _phantom: PhantomData,
        }
    }
}
unsafe fn insert_node<'a, T>(l: *mut *mut Node<'a, T>, item: T, parent: *mut Node<'a, T>)
where
    T: Ord,
{
    if (*l).is_null() {
        let new_tree = Box::new(Node::new(item));
        let new_tree = Box::leak(new_tree);

        new_tree.parent = parent;

        let new_tree: *mut _ = new_tree;
        *l = new_tree;

        return;
    }

    if item < (**l).item {
        let left: *mut *mut _ = &mut ((**l).left);
        insert_node(left, item, *l);
    } else {
        let right: *mut *mut _ = &mut ((**l).right);
        insert_node(right, item, *l);
    }
}

unsafe fn search_node<'a, 'b, T, Q>(l: *const Node<'a, T>, item: &'b Q) -> Option<&'a T>
where
    T: Borrow<Q> + Ord,
    Q: Ord + ?Sized,
{
    if l.is_null() {
        return None;
    }

    match item.cmp((*l).item.borrow()) {
        std::cmp::Ordering::Equal => Some(&(*l).item),
        std::cmp::Ordering::Less => search_node((*l).left, item),
        std::cmp::Ordering::Greater => search_node((*l).right, item),
    }
}

impl<'a, T> BinarySearchTree<'a, T> {
    pub fn new() -> Self {
        Self {
            root: std::ptr::null_mut(),
            _phantom: PhantomData,
        }
    }

    pub fn insert(&mut self, value: T)
    where
        T: Ord,
    {
        unsafe {
            insert_node(&mut self.root, value, std::ptr::null_mut());
        }
    }

    pub fn get<Q>(&'a self, item: &Q) -> Option<&'a T>
    where
        T: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        unsafe { search_node(self.root, item) }
    }

    pub fn contains<Q>(&'a self, item: &Q) -> bool
    where
        T: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.get(item).is_some()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
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
        assert!(!tree.contains(&3));
    }

    #[test]
    fn can_get_refs() {
        let mut tree = BinarySearchTree::new();
        let mut btree = BTreeSet::new();

        tree.insert(String::from("Hello"));
        tree.insert(String::from("World"));
        tree.insert(String::from("How are you?"));
        btree.insert(String::from("How are you?"));

        let q = tree.get("How are you?").unwrap();
        let q2 = btree.get("How are you?").unwrap();

        assert_eq!(q, "How are you?");

        assert_eq!(q, "How are you?");
        assert_eq!(q2, "How are you?");
    }
}
