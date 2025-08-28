use std::{cell::Cell, marker::PhantomData, ops::ControlFlow};

use slab::Slab;

use super::slabref::SlabRef;

/// Head of a list of slab references.
///
/// List layout:
///
/// `None <- HeadGuideNode <-> Node <-> Node <-> ... <-> Node <-> TailGuideNode -> None`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlabListNodeHead {
    pub(crate) prev: usize,
    pub(crate) next: usize,
}

impl SlabListNodeHead {
    pub fn insert_prev(self, prev: Option<usize>) -> Self {
        Self { prev: prev.unwrap_or(usize::MAX), next: self.next }
    }
    pub fn insert_next(self, next: Option<usize>) -> Self {
        Self { prev: self.prev, next: next.unwrap_or(usize::MAX) }
    }

    pub fn get_prev(&self) -> Option<usize> {
        if self.prev == usize::MAX { None } else { Some(self.prev) }
    }
    pub fn get_next(&self) -> Option<usize> {
        if self.next == usize::MAX { None } else { Some(self.next) }
    }
}

/**
 * Error type for `SlabRefList` items.
 */
#[derive(Debug, Clone, Copy)]
pub enum SlabListError {
    Empty,
    InvalidRef,
    InvalidList,
    SelfNotInList(usize),
    NodeIsTailGuide,
    NodeIsHeadGuide,
    PluggedItemAttached(usize),
    UnpluggedItemAttached(usize),
    RepeatedNode(usize),
}

pub trait SlabListNode {
    /**
     * Constructor: Create an empty guide node.
     */
    fn new_guide() -> Self;

    /**
     * Get the node head.
     */
    fn load_node_head(&self) -> SlabListNodeHead;

    /**
     * Set the node head.
     */
    fn store_node_head(&self, node_head: SlabListNodeHead);

    fn get_next(&self) -> Option<usize> {
        self.load_node_head().get_next()
    }
    fn get_prev(&self) -> Option<usize> {
        self.load_node_head().get_prev()
    }
    fn set_next(&self, next: Option<usize>) {
        self.store_node_head(self.load_node_head().insert_next(next));
    }
    fn set_prev(&self, prev: Option<usize>) {
        self.store_node_head(self.load_node_head().insert_prev(prev));
    }

    fn is_guide_head(&self) -> bool {
        self.load_node_head().get_prev().is_none()
    }
    fn is_guide_tail(&self) -> bool {
        self.load_node_head().get_next().is_none()
    }
    fn is_guide(&self) -> bool {
        let node_head = self.load_node_head();
        node_head.get_prev().is_none() || node_head.get_next().is_none()
    }
}

pub trait SlabListNodeRef: SlabRef<RefObject: SlabListNode> {
    fn load_node_head(&self, alloc: &Slab<Self::RefObject>) -> SlabListNodeHead {
        if self.is_null() {
            panic!("SlabRefListNodeRef::load_node_head() called on null reference");
        }
        match self.as_data(alloc).map(SlabListNode::load_node_head) {
            Some(node) => node,
            None => panic!(
                "SlabRefListNodeRef::load_node_head() called on invalid reference {}",
                self.get_handle()
            ),
        }
    }
    fn store_node_head(&self, alloc: &Slab<Self::RefObject>, node_head: SlabListNodeHead) {
        self.as_data(alloc)
            .map(|node| node.store_node_head(node_head))
            .expect("SlabRefListNodeRef::store_node_head() called on invalid reference");
    }

    fn get_next_ref(&self, alloc: &Slab<Self::RefObject>) -> Option<Self> {
        let next = self.load_node_head(alloc).next;
        Self::from_handle(next).to_option()
    }
    fn get_prev_ref(&self, alloc: &Slab<Self::RefObject>) -> Option<Self> {
        let prev = self.load_node_head(alloc).prev;
        Self::from_handle(prev).to_option()
    }

    fn comes_after_node(self, maybe_before: Self, alloc: &Slab<Self::RefObject>) -> bool {
        if self == maybe_before {
            return false;
        }
        let mut curr = self;
        while let Some(prev) = curr.get_prev_ref(alloc) {
            if prev == maybe_before {
                return true;
            }
            curr = prev;
        }
        false
    }
    fn comes_before_node(self, maybe_after: Self, alloc: &Slab<Self::RefObject>) -> bool {
        if self == maybe_after {
            return false;
        }
        let mut curr = self;
        while let Some(next) = curr.get_next_ref(alloc) {
            if next == maybe_after {
                return true;
            }
            curr = next;
        }
        false
    }

    fn on_node_push_next(
        curr: Self,
        next: Self,
        alloc: &Slab<Self::RefObject>,
    ) -> Result<(), SlabListError>;
    fn on_node_push_prev(
        curr: Self,
        prev: Self,
        alloc: &Slab<Self::RefObject>,
    ) -> Result<(), SlabListError>;
    fn on_node_unplug(curr: Self, alloc: &Slab<Self::RefObject>) -> Result<(), SlabListError>;
}

impl SlabListNodeHead {
    pub fn new() -> Self {
        Self { prev: usize::MAX, next: usize::MAX }
    }
}

#[derive(Debug)]
pub struct SlabRefList<T: SlabListNodeRef> {
    pub(crate) _head: T,
    pub(crate) _tail: T,
    pub(crate) _size: Cell<usize>,
    __phantom__: std::marker::PhantomData<T>,
}

impl<T: SlabListNodeRef> SlabRefList<T> {
    pub fn from_slab(slab: &mut Slab<T::RefObject>) -> Self {
        let head = slab.insert(T::RefObject::new_guide());
        let tail = slab.insert(T::RefObject::new_guide());
        slab[head].set_next(Some(tail));
        slab[tail].set_prev(Some(head));
        Self {
            _head: T::from_handle(head),
            _tail: T::from_handle(tail),
            _size: Cell::new(0),
            __phantom__: std::marker::PhantomData,
        }
    }
    pub fn new_guide() -> Self {
        Self {
            _head: T::new_null(),
            _tail: T::new_null(),
            _size: Cell::new(0),
            __phantom__: std::marker::PhantomData,
        }
    }

    pub fn get_front_ref(&self, slab: &Slab<T::RefObject>) -> Option<T> {
        self._head
            .as_data(slab)
            .map(|n| n.get_next())
            .flatten()
            .map(T::from_handle)
    }
    pub fn get_back_ref(&self, slab: &Slab<T::RefObject>) -> Option<T> {
        self._tail
            .as_data(slab)
            .map(|n| n.get_prev())
            .flatten()
            .map(T::from_handle)
    }
    pub fn len(&self) -> usize {
        self._size.get()
    }

    /// Returns the number of nodes in the list, including the head and tail guides.
    /// If the list is valid and empty, it returns 2.
    /// If the list is invalid, it returns 0.
    pub fn n_nodes_with_guide(&self) -> usize {
        let n = self._size.get();
        if self.is_valid() { n + 2 } else { 0 }
    }
    pub fn is_empty(&self) -> bool {
        self._size.get() == 0
    }
    pub fn is_valid(&self) -> bool {
        self._head.is_nonnull() && self._tail.is_nonnull()
    }

    /**
     * Add a node to the `next` position of the current node.
     *
     * SEE the deleted code in the trait `SlabRefListNodeRef` for the original code.
     */
    pub fn node_add_next(
        &self,
        alloc: &Slab<T::RefObject>,
        node_ref: T,
        next_ref: T,
    ) -> Result<(), SlabListError> {
        if node_ref == next_ref {
            return Err(SlabListError::RepeatedNode(node_ref.get_handle()));
        }
        T::on_node_push_next(node_ref.clone(), next_ref.clone(), alloc)?;
        let node = match node_ref.as_data(alloc) {
            Some(node) => node,
            None => return Err(SlabListError::InvalidRef),
        };
        let next_node = match next_ref.as_data(alloc) {
            Some(node) => node,
            None => return Err(SlabListError::InvalidRef),
        };
        let original_next_node = match node
            .get_next()
            .map(|n| T::from_handle(n).as_data(alloc))
            .flatten()
        {
            Some(node) => node,
            None => return Err(SlabListError::NodeIsTailGuide),
        };
        next_node.store_node_head(SlabListNodeHead {
            prev: node_ref.get_handle(),
            next: node.get_next().unwrap_or(usize::MAX),
        });
        node.set_next(Some(next_ref.get_handle()));
        original_next_node.set_prev(Some(next_ref.get_handle()));
        self._size.set(self._size.get() + 1);
        Ok(())
    }

    /**
     * Add a node to the `prev` position of the current node.
     *
     * SEE the deleted code in the trait `SlabRefListNodeRef` for the original code.
     */
    pub fn node_add_prev(
        &self,
        alloc: &Slab<T::RefObject>,
        node_ref: T,
        prev_ref: T,
    ) -> Result<(), SlabListError> {
        if node_ref == prev_ref {
            return Err(SlabListError::RepeatedNode(node_ref.get_handle()));
        }
        T::on_node_push_prev(node_ref.clone(), prev_ref.clone(), alloc)?;
        let node = match node_ref.as_data(alloc) {
            Some(node) => node,
            None => return Err(SlabListError::InvalidRef),
        };
        let prev_node = match prev_ref.as_data(alloc) {
            Some(node) => node,
            None => return Err(SlabListError::InvalidRef),
        };
        let original_prev_node = match node
            .get_prev()
            .map(|n| T::from_handle(n).as_data(alloc))
            .flatten()
        {
            Some(node) => node,
            None => return Err(SlabListError::NodeIsHeadGuide),
        };
        prev_node.store_node_head(SlabListNodeHead {
            prev: node.get_prev().unwrap_or(usize::MAX),
            next: node_ref.get_handle(),
        });
        node.set_prev(Some(prev_ref.get_handle()));
        original_prev_node.set_next(Some(prev_ref.get_handle()));
        self._size.set(self._size.get() + 1);
        Ok(())
    }

    /**
     * Unplug this node from the list.
     *
     * SEE the deleted code in the trait `SlabRefListNodeRef` for the original code.
     */
    pub fn unplug_node(
        &self,
        alloc: &Slab<T::RefObject>,
        node_ref: T,
    ) -> Result<(), SlabListError> {
        if self.is_empty() {
            return Err(SlabListError::Empty);
        }
        T::on_node_unplug(node_ref.clone(), alloc)?;
        let node = match node_ref.as_data(alloc) {
            Some(node) => node,
            None => return Err(SlabListError::InvalidRef),
        };
        let prev_node = match node
            .get_prev()
            .map(|n| T::from_handle(n).as_data(alloc))
            .flatten()
        {
            Some(node) => node,
            None => return Err(SlabListError::NodeIsHeadGuide),
        };
        let next_node = match node
            .get_next()
            .map(|n| T::from_handle(n).as_data(alloc))
            .flatten()
        {
            Some(node) => node,
            None => return Err(SlabListError::NodeIsTailGuide),
        };
        prev_node.set_next(node.get_next());
        next_node.set_prev(node.get_prev());
        self._size.set(self._size.get() - 1);
        Ok(())
    }

    pub fn push_back_ref(
        &self,
        alloc: &Slab<T::RefObject>,
        node_ref: T,
    ) -> Result<(), SlabListError> {
        self.node_add_prev(alloc, self._tail.clone(), node_ref)
    }
    pub fn push_front_ref(
        &self,
        alloc: &Slab<T::RefObject>,
        node_ref: T,
    ) -> Result<(), SlabListError> {
        self.node_add_next(alloc, self._head.clone(), node_ref)
    }
    pub fn pop_back(&self, alloc: &Slab<T::RefObject>) -> Result<T, SlabListError> {
        let tail = &self._tail;
        let prev = tail
            .get_prev_ref(alloc)
            .ok_or(SlabListError::NodeIsHeadGuide)?;
        self.unplug_node(alloc, prev.clone())?;
        Ok(prev)
    }
    pub fn pop_front(&self, alloc: &Slab<T::RefObject>) -> Result<T, SlabListError> {
        let head = &self._head;
        let next = head
            .get_next_ref(alloc)
            .ok_or(SlabListError::NodeIsTailGuide)?;
        self.unplug_node(alloc, head.clone())?;
        Ok(next)
    }

    pub fn view<'a>(&'a self, alloc: &'a Slab<T::RefObject>) -> SlabListView<'a, T> {
        SlabListView {
            _list_range: SlabListRange {
                node_head: self._head.clone(),
                node_tail: self._tail.clone(),
            },
            _slab_alloc: alloc,
        }
    }

    pub fn load_range(&self) -> SlabListRange<T> {
        SlabListRange { node_head: self._head.clone(), node_tail: self._tail.clone() }
    }
    pub fn load_range_and_length(&self) -> (SlabListRange<T>, usize) {
        (self.load_range(), self._size.get())
    }
    pub fn load_range_and_full_node_count(&self) -> (SlabListRange<T>, usize) {
        (self.load_range(), self.n_nodes_with_guide())
    }

    pub unsafe fn unsafe_load_readonly_view(&self) -> Self {
        Self {
            _head: self._head.clone(),
            _tail: self._tail.clone(),
            _size: self._size.clone(),
            __phantom__: PhantomData,
        }
    }

    /// 遍历所有结点, 包括头尾定位结点.
    pub fn forall_nodes(
        &self,
        alloc: &Slab<T::RefObject>,
        mut f: impl FnMut(&T, &T::RefObject) -> ControlFlow<()>,
    ) {
        if !self.is_valid() {
            return;
        }
        let mut current = self._head.clone();
        loop {
            let data = match current.as_data(alloc) {
                Some(data) => data,
                None => break,
            };
            if let ControlFlow::Break(_) = f(&current, data) {
                break;
            }
            // 到达尾结点后停止
            if current == self._tail {
                break;
            }
            let next = match current.get_next_ref(alloc) {
                Some(n) => n,
                None => break,
            };
            current = next;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlabListRange<T: SlabListNodeRef> {
    pub node_head: T,
    pub node_tail: T,
}

impl<T: SlabListNodeRef> SlabListRange<T> {
    pub fn view<'a>(&'a self, slab: &'a Slab<T::RefObject>) -> SlabListView<'a, T> {
        SlabListView { _list_range: self.clone(), _slab_alloc: slab }
    }

    pub fn calc_length(&self, slab: &Slab<T::RefObject>) -> usize {
        let mut length = 0;
        let mut current = self.node_head.get_next_ref(slab);
        while let Some(node) = current {
            // 如果遇到 tail guide 节点，停止计数
            if node == self.node_tail {
                break;
            }
            length += 1;
            current = node.get_next_ref(slab);
        }
        length
    }
}

pub struct SlabListView<'a, T: SlabListNodeRef> {
    pub(crate) _list_range: SlabListRange<T>,
    pub(crate) _slab_alloc: &'a Slab<T::RefObject>,
}

pub struct SlabListIterator<'a, T: SlabListNodeRef> {
    _current: T,
    _node_tail: T,
    _alloc: &'a Slab<T::RefObject>,
}

impl<'a, T: SlabListNodeRef> Iterator for SlabListIterator<'a, T> {
    type Item = (T, &'a T::RefObject);

    fn next(&mut self) -> Option<Self::Item> {
        if self._current == self._node_tail || self._current.is_null() {
            return None;
        }
        let current_data = self._current.as_data(self._alloc)?;
        let next = T::from_handle(current_data.get_next()?);
        let item = (self._current.clone(), current_data);
        self._current = next;
        Some(item)
    }
}

impl<'a, T: SlabListNodeRef> IntoIterator for SlabListView<'a, T> {
    type Item = (T, &'a T::RefObject);
    type IntoIter = SlabListIterator<'a, T>;

    fn into_iter(self) -> SlabListIterator<'a, T> {
        let list_range = &self._list_range;
        let curr = if list_range.node_head.is_null() {
            list_range.node_tail.clone()
        } else if list_range.node_head == list_range.node_tail {
            list_range.node_tail.clone()
        } else {
            list_range
                .node_head
                .get_next_ref(self._slab_alloc)
                .unwrap_or(list_range.node_tail.clone())
        };
        SlabListIterator {
            _current: curr,
            _node_tail: list_range.node_tail.clone(),
            _alloc: self._slab_alloc,
        }
    }
}

impl<T> std::fmt::Debug for SlabListView<'_, T>
where
    T: SlabListNodeRef<RefObject: std::fmt::Debug>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(
                self._list_range
                    .view(self._slab_alloc)
                    .into_iter()
                    .map(|(_, data)| data),
            )
            .finish()
    }
}

pub type SlabListRes = Result<(), SlabListError>;

#[cfg(test)]
mod testing {
    use super::*;
    use slab::Slab;

    #[derive(Debug)]
    struct TestNode {
        node_head: Cell<SlabListNodeHead>,
        number: usize,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct TestNodeRef(usize);

    impl SlabListNode for TestNode {
        fn new_guide() -> Self {
            Self { node_head: Cell::new(SlabListNodeHead::new()), number: 0 }
        }
        fn load_node_head(&self) -> SlabListNodeHead {
            self.node_head.get()
        }
        fn store_node_head(&self, node_head: SlabListNodeHead) {
            self.node_head.set(node_head);
        }
    }

    impl TestNode {
        fn new(number: usize) -> Self {
            Self { node_head: Cell::new(SlabListNodeHead::new()), number }
        }
    }

    impl SlabRef for TestNodeRef {
        type RefObject = TestNode;

        fn from_handle(handle: usize) -> Self {
            Self(handle)
        }

        fn get_handle(&self) -> usize {
            self.0
        }
    }

    impl SlabListNodeRef for TestNodeRef {
        fn on_node_push_next(_: Self, _: Self, _: &Slab<TestNode>) -> Result<(), SlabListError> {
            Ok(())
        }

        fn on_node_push_prev(_: Self, _: Self, _: &Slab<TestNode>) -> Result<(), SlabListError> {
            Ok(())
        }

        fn on_node_unplug(_: Self, _: &Slab<TestNode>) -> Result<(), SlabListError> {
            Ok(())
        }
    }

    #[allow(dead_code)]
    fn test_list_from_vec(
        alloc: &mut Slab<<TestNodeRef as SlabRef>::RefObject>,
        items: Vec<usize>,
    ) -> SlabRefList<TestNodeRef> {
        let list = SlabRefList::from_slab(alloc);
        for item in items {
            let node_ref = alloc.insert(TestNode::new(item));
            list.node_add_prev(alloc, list._tail, TestNodeRef(node_ref))
                .unwrap();
        }
        list
    }

    #[allow(dead_code)]
    fn print_test_list(list: &SlabRefList<TestNodeRef>, slab: &Slab<TestNode>) {
        print!("List({} elems): [ ", list.len());
        for (_, i) in list.view(slab) {
            print!("{}, ", i.number);
        }
        println!("]");
    }

    #[test]
    fn slab_node_test() {
        let mut slab = Slab::new();
        let list = test_list_from_vec(&mut slab, vec![1, 2, 3, 4, 5]);
        assert_eq!(list.len(), 5);
        print_test_list(&list, &slab);
    }

    #[test]
    fn slab_push_pop_test() {
        let mut slab = Slab::new();
        let list = test_list_from_vec(&mut slab, vec![1, 2, 3, 4, 5]);

        assert_eq!(list.len(), 5);

        for i in 6..=10 {
            let test_ref = TestNodeRef(slab.insert(TestNode::new(i)));
            list.push_back_ref(&mut slab, test_ref).unwrap();
        }
        assert_eq!(list.len(), 10);
        print_test_list(&list, &slab);

        for i in 0..5 {
            let node = list.pop_back(&slab).unwrap();
            assert_eq!(node.as_data(&slab).unwrap().number, 10 - i);
            node.free_from_alloc(&mut slab);
        }
        assert_eq!(list.len(), 5);
        print_test_list(&list, &slab);
    }

    #[test]
    fn calc_length_test() {
        let mut slab = Slab::new();

        // 测试空列表
        let empty_list = test_list_from_vec(&mut slab, vec![]);
        let range = empty_list.load_range();
        assert_eq!(range.calc_length(&slab), 0);
        assert_eq!(range.calc_length(&slab), empty_list.len());

        // 测试有元素的列表
        let list = test_list_from_vec(&mut slab, vec![1, 2, 3, 4, 5]);
        let range = list.load_range();
        assert_eq!(range.calc_length(&slab), 5);
        assert_eq!(range.calc_length(&slab), list.len());

        // 添加更多元素后测试
        for i in 6..=10 {
            let test_ref = TestNodeRef(slab.insert(TestNode::new(i)));
            list.push_back_ref(&mut slab, test_ref).unwrap();
        }
        let range = list.load_range();
        assert_eq!(range.calc_length(&slab), 10);
        assert_eq!(range.calc_length(&slab), list.len());

        println!("calc_length test passed!");
    }
}
