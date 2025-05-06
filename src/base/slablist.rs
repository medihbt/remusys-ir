use std::cell::Cell;

use slab::Slab;

use super::slabref::SlabRef;

/**
 * Head of a list of slab references.
 * 
 * List layout:
 * 
 * ```
 * None <- [Head Guide Node] <-> [Node] <-> [Node] <-> ... <-> [Node] <-> [Tail Guide Node] -> None
 * ```
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlabRefListNodeHead {
    pub(crate) prev: Option<usize>,
    pub(crate) next: Option<usize>,
}

impl SlabRefListNodeHead {
    pub fn insert_prev(self, prev: Option<usize>) -> Self {
        Self {
            prev: prev,
            next: self.next,
        }
    }
    pub fn insert_next(self, next: Option<usize>) -> Self {
        Self {
            prev: self.prev,
            next: next,
        }
    }
}

/**
 * Error type for `SlabRefList` items.
 */
#[derive(Debug, Clone)]
pub enum SlabRefListError {
    Empty,
    InvalidRef,
    NodeIsTailGuide, NodeIsHeadGuide,
    PluggedItemAttached  (usize),
    UnpluggedItemAttached(usize),
}

pub trait SlabRefListNode {
    /**
     * Constructor: Create an empty guide node.
     */
    fn new_guide() -> Self;

    /**
     * Get the node head.
     */
    fn load_node_head(&self) -> SlabRefListNodeHead;

    /**
     * Set the node head.
     */
    fn store_node_head(&self, node_head: SlabRefListNodeHead);

    fn get_next(&self) -> Option<usize> { self.load_node_head().next }
    fn get_prev(&self) -> Option<usize> { self.load_node_head().prev }
    fn set_next(&self, next: Option<usize>) {
        self.store_node_head(
            self.load_node_head().insert_next(next)
        );
    }
    fn set_prev(&self, prev: Option<usize>) {
        self.store_node_head(
            self.load_node_head().insert_prev(prev)
        );
    }

    fn is_guide_head(&self) -> bool {
        self.load_node_head().prev.is_none()
    }
    fn is_guide_tail(&self) -> bool {
        self.load_node_head().next.is_none()
    }
    fn is_guide(&self) -> bool {
        let node_head = self.load_node_head();
        node_head.prev.is_none() || node_head.next.is_none()
    }
}

pub trait SlabRefListNodeRef: SlabRef<Item: SlabRefListNode> {
    fn load_node_head(&self, alloc: &Slab<Self::Item>) -> SlabRefListNodeHead {
        self.to_slabref(alloc)
            .map(SlabRefListNode::load_node_head)
            .expect("SlabRefListNodeRef::load_node_head() called on invalid reference")
    }
    fn store_node_head(&self, alloc: &Slab<Self::Item>, node_head: SlabRefListNodeHead) {
        self.to_slabref(alloc)
            .map(|node| node.store_node_head(node_head))
            .expect("SlabRefListNodeRef::store_node_head() called on invalid reference");
    }

    fn get_next_ref(&self, alloc: &Slab<Self::Item>) -> Option<Self> {
        self.load_node_head(alloc)
            .next
            .map(Self::from_handle)
    }
    fn get_prev_ref(&self, alloc: &Slab<Self::Item>) -> Option<Self> {
        self.load_node_head(alloc)
            .prev
            .map(Self::from_handle)
    }
}

impl SlabRefListNodeHead {
    pub fn new() -> Self {
        Self {
            prev: None,
            next: None,
        }
    }
}

#[derive(Debug)]
pub struct SlabRefList<T: SlabRefListNodeRef> {
    pub(crate) _head: T,
    pub(crate) _tail: T,
    pub(crate) _size: usize,
    __phantom__: std::marker::PhantomData<T>,
}

impl<T: SlabRefListNodeRef> SlabRefList<T> {
    pub fn from_slab(slab: &mut Slab<T::Item>) -> Self {
        let head = slab.insert(T::Item::new_guide());
        let tail = slab.insert(T::Item::new_guide());
        slab[head].set_next(Some(tail));
        slab[tail].set_prev(Some(head));
        Self {
            _head: T::from_handle(head),
            _tail: T::from_handle(tail),
            _size: 0,
            __phantom__: std::marker::PhantomData,
        }
    }

    pub fn get_front_ref(&self, slab: &Slab<T::Item>) -> Option<usize> {
        self._head
            .to_slabref(slab)
            .map(|n| n.get_next())
            .flatten()
    }
    pub fn get_back_ref(&self, slab: &Slab<T::Item>) -> Option<usize> {
        self._tail
            .to_slabref(slab)
            .map(|n| n.get_prev())
            .flatten()
    }
    pub fn get_size(&self) -> usize { self._size }
    pub fn is_empty(&self) -> bool  { self._size == 0 }

    /**
     * Add a node to the `next` position of the current node.
     * 
     * SEE the deleted code in the trait `SlabRefListNodeRef` for the original code.
     */
    pub fn node_add_next(&mut self, alloc: &Slab<T::Item>, node_ref: T, next_ref: T) -> Result<(), SlabRefListError> {
        let node = match node_ref.to_slabref(alloc) {
            Some(node) => node,
            None => return Err(SlabRefListError::InvalidRef),
        };
        let next_node = match next_ref.to_slabref(alloc) {
            Some(node) => node,
            None => return Err(SlabRefListError::InvalidRef),
        };
        let original_next_node = match node.get_next()
            .map(|n| T::from_handle(n).to_slabref(alloc))
            .flatten() {
            Some(node) => node,
            None => return Err(SlabRefListError::NodeIsTailGuide),
        };
        next_node.store_node_head(
            SlabRefListNodeHead {
                prev: Some(node_ref.get_handle()),
                next: node.get_next()
            }
        );
        node.set_next(Some(next_ref.get_handle()));
        original_next_node.set_prev(Some(next_ref.get_handle()));
        self._size += 1;
        Ok(())
    }

    /**
     * Add a node to the `prev` position of the current node.
     * 
     * SEE the deleted code in the trait `SlabRefListNodeRef` for the original code.
     */
    pub fn node_add_prev(&mut self, alloc: &Slab<T::Item>, node_ref: T, prev_ref: T) -> Result<(), SlabRefListError> {
        let node = match node_ref.to_slabref(alloc) {
            Some(node) => node,
            None => return Err(SlabRefListError::InvalidRef),
        };
        let prev_node = match prev_ref.to_slabref(alloc) {
            Some(node) => node,
            None => return Err(SlabRefListError::InvalidRef),
        };
        let original_prev_node = match node.get_prev()
            .map(|n| T::from_handle(n).to_slabref(alloc))
            .flatten() {
            Some(node) => node,
            None => return Err(SlabRefListError::NodeIsHeadGuide),
        };
        prev_node.store_node_head(
            SlabRefListNodeHead {
                prev: node.get_prev(),
                next: Some(node_ref.get_handle())
            }
        );
        node.set_prev(Some(prev_ref.get_handle()));
        original_prev_node.set_next(Some(prev_ref.get_handle()));
        self._size += 1;
        Ok(())
    }

    /**
     * Unplug this node from the list.
     * 
     * SEE the deleted code in the trait `SlabRefListNodeRef` for the original code.
     */
    pub fn unplug_node(&mut self, alloc: &Slab<T::Item>, node_ref: T) -> Result<(), SlabRefListError> {
        let node = match node_ref.to_slabref(alloc) {
            Some(node) => node,
            None => return Err(SlabRefListError::InvalidRef),
        };
        let prev_node = match node.get_prev()
            .map(|n| T::from_handle(n).to_slabref(alloc))
            .flatten() {
            Some(node) => node,
            None => return Err(SlabRefListError::NodeIsHeadGuide),
        };
        let next_node = match node.get_next()
            .map(|n| T::from_handle(n).to_slabref(alloc))
            .flatten() {
            Some(node) => node,
            None => return Err(SlabRefListError::NodeIsTailGuide),
        };
        prev_node.set_next(node.get_next());
        next_node.set_prev(node.get_prev());
        self._size -= 1;
        Ok(())
    }

    pub fn push_back_ref(&mut self, alloc: &Slab<T::Item>, node_ref: T) -> Result<(), SlabRefListError> {
        self.node_add_prev(alloc, self._tail.clone(), node_ref)
    }
    pub fn push_front_ref(&mut self, alloc: &Slab<T::Item>, node_ref: T) -> Result<(), SlabRefListError> {
        self.node_add_next(alloc, self._head.clone(), node_ref)
    }
    pub fn push_back_value(&mut self, alloc: &mut Slab<T::Item>, value: T::Item) -> Result<T, SlabRefListError> {
        let node_ref = T::from_handle(alloc.insert(value));
        self.push_back_ref(alloc, node_ref.clone()).unwrap();
        Ok(node_ref)
    }
    pub fn push_front_value(&mut self, alloc: &mut Slab<T::Item>, value: T::Item) -> Result<T, SlabRefListError> {
        let node_ref = T::from_handle(alloc.insert(value));
        self.push_front_ref(alloc, node_ref.clone()).unwrap();
        Ok(node_ref)
    }
    pub fn pop_back(&mut self, alloc: &Slab<T::Item>) -> Result<T, SlabRefListError> {
        let tail = &self._tail;
        let prev = tail.get_prev_ref(alloc).ok_or(SlabRefListError::NodeIsHeadGuide)?;
        self.unplug_node(alloc, prev.clone())?;
        Ok(prev)
    }
    pub fn pop_front(&mut self, alloc: &Slab<T::Item>) -> Result<T, SlabRefListError> {
        let head = &self._head;
        let next = head.get_next_ref(alloc).ok_or(SlabRefListError::NodeIsTailGuide)?;
        self.unplug_node(alloc, head.clone())?;
        Ok(next)
    }

    pub fn view<'a>(&'a self, alloc: &'a Slab<T::Item>) -> SlabRefListView<'a, T> {
        SlabRefListView {
            _list: self,
            _slab: alloc,
        }
    }
}

pub struct SlabRefListView<'a, T: SlabRefListNodeRef> {
    pub(crate) _list: &'a SlabRefList<T>,
    pub(crate) _slab: &'a Slab<T::Item>,
}

pub struct SlabRefListIterator<'a, T: SlabRefListNodeRef> {
    pub(crate) _current: Option<usize>,
    pub(crate) _slab: &'a Slab<T::Item>,
}

impl<T: SlabRefListNodeRef> Iterator for SlabRefListIterator<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self._current?;
        let next = self._slab[current].get_next()?;
        self._current = Some(next);
        Some(T::from_handle(current))
    }
}

impl<'a, T> IntoIterator for SlabRefListView<'a, T>
where
    T: SlabRefListNodeRef,
{
    type Item = T;
    type IntoIter = SlabRefListIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        let current = self._list
            ._head
            .get_next_ref(self._slab)
            .expect("Head node should have a next node")
            .get_handle();
        SlabRefListIterator {
            _current: Some(current),
            _slab: &self._slab,
        }
    }
}

impl<T> std::fmt::Debug for SlabRefListView<'_, T>
where
    T: SlabRefListNodeRef<Item: std::fmt::Debug>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(
                self._list
                    .view(self._slab)
                    .into_iter()
                    .map(|node| node.to_slabref(self._slab).unwrap())
            )
            .finish()
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    use slab::Slab;

    #[derive(Debug)]
    struct TestNode {
        node_head: Cell<SlabRefListNodeHead>,
        number: usize,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct TestNodeRef(usize);

    impl SlabRefListNode for TestNode {
        fn new_guide() -> Self {
            Self {
                node_head: Cell::new(SlabRefListNodeHead { prev: None, next: None }),
                number: 0,
            }
        }
        fn load_node_head(&self) -> SlabRefListNodeHead {
            self.node_head.get()
        }
        fn store_node_head(&self, node_head: SlabRefListNodeHead) {
            self.node_head.set(node_head);
        }
    }

    impl TestNode {
        fn new(number: usize) -> Self {
            Self {
                node_head: Cell::new(SlabRefListNodeHead { prev: None, next: None }),
                number,
            }
        }
    }

    impl SlabRef for TestNodeRef {
        type Item = TestNode;
    
        fn from_handle(handle: usize) -> Self {
            Self(handle)
        }
    
        fn get_handle (&self) -> usize {
            self.0
        }
    }

    impl SlabRefListNodeRef for TestNodeRef {}

    #[allow(dead_code)]
    fn test_list_from_vec(alloc: &mut Slab<<TestNodeRef as SlabRef>::Item>, items: Vec<usize>) -> SlabRefList<TestNodeRef> {
        let mut list = SlabRefList::from_slab(alloc);
        for item in items {
            let node_ref = alloc.insert(TestNode::new(item));
            list.node_add_prev(alloc, list._tail, TestNodeRef(node_ref)).unwrap();
        }
        list
    }

    #[test]
    fn slab_node_test() {
        let mut slab = Slab::new();
        let list = test_list_from_vec(&mut slab, vec![1, 2, 3, 4, 5]);
        assert_eq!(list.get_size(), 5);
        println!("List: {:#?}", list.view(&slab));
    }

    #[test]
    fn slab_push_pop_test() {
        let mut slab = Slab::new();
        let mut list = test_list_from_vec(&mut slab, vec![1, 2, 3, 4, 5]);

        assert_eq!(list.get_size(), 5);

        for i in 6..=10 {
            list.push_back_value(&mut slab, TestNode::new(i)).unwrap();
        }
        assert_eq!(list.get_size(), 10);
        println!("List: {:#?}", list.view(&slab));

        for i in 0..5 {
            let node = list.pop_back(&slab).unwrap();
            assert_eq!(node.to_slabref(&slab).unwrap().number, 10 - i);
            slab.remove(node.get_handle());
        }
        assert_eq!(list.get_size(), 5);
        println!("List: {:#?}", list.view(&slab));
    }
}