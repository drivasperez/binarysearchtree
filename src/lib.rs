use std::{borrow::Borrow, marker::PhantomData};

pub struct BinarySearchTree<'a, T> {
    root: *mut Node<'a, T>,
    _phantom: PhantomData<&'a mut T>,
}

struct Node<'a, T> {
    item: T,
    parent: *mut Node<'a, T>,
    left: *mut Node<'a, T>,
    right: *mut Node<'a, T>,
    _phantom: PhantomData<&'a mut T>,
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

    pub fn item(&'a self) -> &'a T {
        &self.item
    }
}
unsafe fn insert_node<'a, T>(l: *mut *mut Node<'a, T>, item: T, parent: *mut Node<'a, T>)
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

unsafe fn search_node<'a, 'b, T, Q>(l: *mut Node<'a, T>, item: &'b Q) -> Option<*mut Node<'a, T>>
where
    T: Borrow<Q> + Ord,
    Q: Ord + ?Sized,
{
    if l.is_null() {
        return None;
    }

    match item.cmp((*l).item.borrow()) {
        std::cmp::Ordering::Equal => Some(l),
        std::cmp::Ordering::Less => search_node((*l).left, item),
        std::cmp::Ordering::Greater => search_node((*l).right, item),
    }
}

unsafe fn delete_node<'a, T>(node: *mut *mut Node<'a, T>)
where
    T: Ord,
{
    if node.is_null() {
        return;
    }

    let node_ref = &mut **node;

    match (node_ref.left.is_null(), node_ref.right.is_null()) {
        (true, true) => {
            // Node has no children, so we just deallocate it.
            drop(node_ref);
            let _ = Box::from_raw(*node);
        }
        (true, false) => {
            // Node has one child (right), link it to parent.
            let parent = node_ref.parent;
            let child = node_ref.right;
            // Set child's parent to node's parent.
            (*child).parent = parent;
            // Free this node.
            let _ = Box::from_raw(*node);
            // Set this node pointer to point at child.
            *node = child;
        }
        (false, true) => {
            // Node has one child (left), link it to parent.
            let parent = node_ref.parent;
            let child = node_ref.left;
            // Set child's parent to node's parent.
            (*child).parent = parent;
            // Free this node.
            let _ = Box::from_raw(*node);
            // Set this node pointer to point at child.
            *node = child;
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

            (*node_ref.parent).left = std::ptr::null_mut();

            // Turn next_biggest back into a box
            let next_biggest = Box::from_raw(next_biggest);
            node_ref.item = (next_biggest).item;
        }
    }
}

unsafe fn find_minimum<'a, T>(t: *const Node<'a, T>) -> Option<&'a T>
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

unsafe fn find_maximum<'a, T>(t: *const Node<'a, T>) -> Option<&'a T>
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
        unsafe {
            let node = search_node(self.root, item);
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

    pub fn delete<Q>(&self, item: &Q)
    where
        T: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        unsafe {
            let node = search_node(self.root, item);
            if let Some(mut ptr) = node {
                delete_node(&mut ptr as *mut _);
            }
        }
    }
}
