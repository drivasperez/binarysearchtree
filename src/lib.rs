#![feature(dropck_eyepatch)]

use std::{borrow::Borrow, marker::PhantomData};

pub struct BinarySearchTree<T> {
    root: *mut Node<T>,
    _phantom: PhantomData<T>,
}

unsafe impl<#[may_dangle] T> Drop for BinarySearchTree<T> {
    fn drop(&mut self) {
        unsafe {
            dispose_node(self.root);
        }
    }
}

struct Node<T> {
    item: T,
    parent: *mut Node<T>,
    left: *mut Node<T>,
    right: *mut Node<T>,
    _phantom: PhantomData<T>,
}

impl<'a, T> Node<T> {
    pub fn new(item: T) -> Self {
        Self {
            item,
            parent: std::ptr::null_mut(),
            left: std::ptr::null_mut(),
            right: std::ptr::null_mut(),
            _phantom: PhantomData,
        }
    }

    pub fn item(&'a self) -> &'a T {
        &self.item
    }
}

unsafe fn dispose_node<T>(l: *mut Node<T>) {
    if l.is_null() {
        return;
    }

    let node_ref = &mut *l;
    match (node_ref.left.is_null(), node_ref.right.is_null()) {
        (true, true) => {
            // node is a leaf, just drop it.
            drop(node_ref);
            let _ = Box::from_raw(l);
        }
        (true, false) => {
            dispose_node(node_ref.right);
            drop(node_ref);
            let _ = Box::from_raw(l);
        }
        (false, true) => {
            dispose_node(node_ref.left);
            drop(node_ref);
            let _ = Box::from_raw(l);
        }
        (false, false) => {
            dispose_node(node_ref.left);
            dispose_node(node_ref.right);
            drop(node_ref);
            let _ = Box::from_raw(l);
        }
    }
}

unsafe fn insert_node<'a, T>(l: *mut *mut Node<T>, item: T, parent: *mut Node<T>)
where
    T: Ord,
{
    if (*l).is_null() {
        let mut new_tree = Box::new(Node::new(item));
        new_tree.parent = parent;
        let new_tree = Box::into_raw(new_tree);

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

unsafe fn search_node<'a, 'b, T, Q>(
    l: *mut Node<T>,
    item: &'b Q,
    called_once: bool,
) -> (bool, Option<*mut Node<T>>)
where
    T: Borrow<Q> + Ord,
    Q: Ord + ?Sized,
{
    if l.is_null() {
        return (called_once, None);
    }

    match item.cmp((*l).item.borrow()) {
        std::cmp::Ordering::Equal => (called_once, Some(l)),
        std::cmp::Ordering::Less => search_node((*l).left, item, false),
        std::cmp::Ordering::Greater => search_node((*l).right, item, false),
    }
}

unsafe fn delete_node<'a, T>(node: *mut *mut Node<T>) -> bool
where
    T: Ord,
{
    if node.is_null() {
        return false;
    }

    let node_ref = &mut **node;

    match (node_ref.left.is_null(), node_ref.right.is_null()) {
        (true, true) => {
            // Node has no children, so we just deallocate it.
            drop(node_ref);
            let _ = Box::from_raw(*node);
            true
        }
        (true, false) => {
            // Node has one child (right), copy child to node.
            // Take ownership of right.
            let child = Box::from_raw(node_ref.right);

            node_ref.right = child.right;
            node_ref.left = child.left;
            node_ref.item = child.item;

            false
            // child is dropped here
        }
        (false, true) => {
            // Node has one child (left), copy child to node.
            // Take ownership of left.
            let child = Box::from_raw(node_ref.left);

            node_ref.right = child.right;
            node_ref.left = child.left;
            node_ref.item = child.item;

            false
            // child is dropped here
        }
        (false, false) => {
            // Node has two children.
            // Solution is to replace this node's value with the left-most descendant of the right child.
            // i.e., the smallest node that is larger than this one.
            // Then delete that node.
            let mut next_biggest = (&mut *node_ref).right;
            while !(&*next_biggest).left.is_null() {
                next_biggest = (&*next_biggest).left;
            }

            // Turn next_biggest back into a box
            let next_biggest = Box::from_raw(next_biggest);
            (*next_biggest.parent).left = std::ptr::null_mut();
            node_ref.left = next_biggest.left;
            node_ref.right = next_biggest.right;
            node_ref.item = (next_biggest).item;

            false
        }
    }
}

unsafe fn find_minimum<'a, T>(t: *const Node<T>) -> Option<&'a T>
where
    T: Ord,
{
    if t.is_null() {
        return None;
    }

    let mut min = &*t;

    while !min.left.is_null() {
        min = &*min.left;
    }

    Some(&min.item)
}

unsafe fn find_maximum<'a, T>(t: *const Node<T>) -> Option<&'a T>
where
    T: Ord,
{
    if t.is_null() {
        return None;
    }

    let mut max = &*t;

    while !max.right.is_null() {
        max = &*max.right;
    }

    Some(&max.item)
}

impl<'a, T> BinarySearchTree<T> {
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
        unsafe {
            let (_, node) = search_node(self.root, item, true);
            node.map(|ptr| (&*ptr).item())
        }
    }

    pub fn contains<Q>(&'a self, item: &Q) -> bool
    where
        T: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.get(item).is_some()
    }

    pub fn min(&self) -> Option<&T>
    where
        T: Ord,
    {
        unsafe { find_minimum(self.root) }
    }

    pub fn max(&self) -> Option<&T>
    where
        T: Ord,
    {
        unsafe { find_maximum(self.root) }
    }

    pub fn delete<Q>(&mut self, item: &Q)
    where
        T: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        unsafe {
            let (is_root, node) = search_node(self.root, item, true);
            if let Some(mut ptr) = node {
                let was_leaf = delete_node(&mut ptr as *mut _);
                // if the deleted node was the last node, change root to NULL.
                // Only the last node if it was the root node, and it had no children.

                if is_root && was_leaf {
                    self.root = std::ptr::null_mut();
                }
            }
        }
    }
}
