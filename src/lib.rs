#![feature(dropck_eyepatch)]

use std::{borrow::Borrow, marker::PhantomData, ptr::NonNull};

struct Node<T> {
    item: T,
    parent: Option<NonNull<Node<T>>>,
    left: Option<NonNull<Node<T>>>,
    right: Option<NonNull<Node<T>>>,
}

pub struct BinarySearchTree<T> {
    root: Option<NonNull<Node<T>>>,
    _marker: PhantomData<Node<T>>,
}

unsafe impl<#[may_dangle] T> Drop for BinarySearchTree<T> {
    fn drop(&mut self) {
        if let Some(root) = self.root {
            unsafe {
                dispose_node(root);
            }
        }
    }
}

impl<'a, T> Node<T> {
    pub fn new(item: T) -> Self {
        Self {
            item,
            parent: None,
            left: None,
            right: None,
        }
    }

    pub fn item(&'a self) -> &'a T {
        &self.item
    }
}

unsafe fn dispose_node<T>(l: NonNull<Node<T>>) {
    let node_ref = l.as_ref();
    match (node_ref.left, node_ref.right) {
        (None, None) => {
            // node is a leaf, just drop it.
            drop(node_ref);
            let _ = Box::from_raw(l.as_ptr());
        }
        (None, Some(right)) => {
            dispose_node(right);
            drop(node_ref);
            let _ = Box::from_raw(l.as_ptr());
        }
        (Some(left), None) => {
            dispose_node(left);
            drop(node_ref);
            let _ = Box::from_raw(l.as_ptr());
        }
        (Some(left), Some(right)) => {
            dispose_node(left);
            dispose_node(right);
            drop(node_ref);
            let _ = Box::from_raw(l.as_ptr());
        }
    }
}

unsafe fn insert_node<'a, T>(
    l: &mut Option<NonNull<Node<T>>>,
    item: T,
    parent: Option<NonNull<Node<T>>>,
) where
    T: Ord,
{
    if let Some(mut leaf) = *l {
        let leaf = leaf.as_mut();
        if item < leaf.item {
            let left = &mut leaf.left;
            insert_node(left, item, *l);
        } else {
            let right = &mut leaf.right;
            insert_node(right, item, *l);
        }
    } else {
        let mut new_tree = Box::new(Node::new(item));
        new_tree.parent = parent;
        let new_tree = Box::into_raw(new_tree);
        let new_tree = NonNull::new_unchecked(new_tree);

        *l = Some(new_tree);
    }
}

unsafe fn search_node<T, Q>(
    l: Option<NonNull<Node<T>>>,
    item: &'_ Q,
    called_once: bool,
) -> (bool, Option<NonNull<Node<T>>>)
where
    T: Borrow<Q> + Ord,
    Q: Ord + ?Sized,
{
    if let Some(mut leaf) = l {
        let leaf_ref = leaf.as_mut();
        match item.cmp(leaf_ref.item.borrow()) {
            std::cmp::Ordering::Equal => (called_once, Some(leaf)),
            std::cmp::Ordering::Less => search_node(leaf_ref.left, item, false),
            std::cmp::Ordering::Greater => search_node(leaf_ref.right, item, false),
        }
    } else {
        (called_once, None)
    }
}

unsafe fn delete_node<T>(node: Option<&mut NonNull<Node<T>>>) -> bool
where
    T: Ord,
{
    if let Some(node) = node {
        let node_ref = node.as_mut();

        match (node_ref.left, node_ref.right) {
            (None, None) => {
                // Node has no children, so we just deallocate it.
                let _ = Box::from_raw(node.as_mut());
                true
            }
            (None, Some(mut right)) => {
                // Node has one child (right), copy child to node.
                // Take ownership of right.
                let child = Box::from_raw(right.as_mut());

                node_ref.right = child.right;
                node_ref.left = child.left;
                node_ref.item = child.item;

                false
                // child is dropped here
            }
            (Some(mut left), None) => {
                // Node has one child (left), copy child to node.
                // Take ownership of left.
                let child = Box::from_raw(left.as_mut());

                node_ref.right = child.right;
                node_ref.left = child.left;
                node_ref.item = child.item;

                false
                // child is dropped here
            }
            (Some(_), Some(right)) => {
                // Node has two children.
                // Solution is to replace this node's value with the left-most descendant of the right child.
                // i.e., the smallest node that is larger than this one.
                // Then delete that node.
                let mut next_biggest = right;
                while let Some(left) = next_biggest.as_ref().left {
                    next_biggest = left;
                }

                // Turn next_biggest back into a box
                let next_biggest = Box::from_raw(next_biggest.as_mut());
                if let Some(mut parent) = next_biggest.parent {
                    parent.as_mut().left = None;
                }
                node_ref.left = next_biggest.left;
                node_ref.right = next_biggest.right;
                node_ref.item = (next_biggest).item;

                false
            }
        }
    } else {
        false
    }
}

unsafe fn find_minimum<'a, T>(t: Option<NonNull<Node<T>>>) -> Option<&'a T>
where
    T: Ord,
{
    if let Some(t) = t {
        let mut min = t;

        loop {
            match min.as_ref().left {
                None => break,
                Some(left) => {
                    min = left;
                }
            }
        }

        Some(&min.as_ref().item)
    } else {
        None
    }
}

unsafe fn find_maximum<'a, T>(t: Option<NonNull<Node<T>>>) -> Option<&'a T>
where
    T: Ord,
{
    if let Some(t) = t {
        let mut max = t;

        loop {
            match max.as_ref().right {
                None => break,
                Some(right) => {
                    max = right;
                }
            }
        }

        Some(&max.as_ref().item)
    } else {
        None
    }
}

impl<T> Default for BinarySearchTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> BinarySearchTree<T> {
    pub fn new() -> Self {
        Self {
            root: None,
            _marker: PhantomData,
        }
    }

    pub fn insert(&mut self, value: T)
    where
        T: Ord,
    {
        unsafe {
            if let Some(root) = self.root {
                insert_node(&mut Some(root), value, None);
            } else {
                // Safety: Box::into_raw is never null.
                let root_ptr = NonNull::new_unchecked(Box::into_raw(Box::new(Node::new(value))));
                self.root = Some(root_ptr);
            }
        }
    }

    pub fn get<Q>(&'a self, item: &Q) -> Option<&'a T>
    where
        T: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        if let Some(root) = self.root {
            unsafe {
                let (_, node) = search_node(Some(root), item, true);
                node.map(|ptr| ptr.as_ref().item())
            }
        } else {
            None
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
        if let Some(root) = self.root {
            unsafe { find_minimum(Some(root)) }
        } else {
            None
        }
    }

    pub fn max(&self) -> Option<&T>
    where
        T: Ord,
    {
        if let Some(root) = self.root {
            unsafe { find_maximum(Some(root)) }
        } else {
            None
        }
    }

    pub fn delete<Q>(&mut self, item: &Q)
    where
        T: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        if let Some(root) = self.root {
            unsafe {
                let (is_root, node) = search_node(Some(root), item, true);
                if let Some(mut ptr) = node {
                    let was_leaf = delete_node(Some(&mut ptr));
                    // if the deleted node was the last node, change root to NULL.
                    // Only the last node if it was the root node, and it had no children.

                    if is_root && was_leaf {
                        self.root = None;
                    }
                }
            }
        }
    }
}
