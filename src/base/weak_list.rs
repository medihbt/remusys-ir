use std::{
    fmt::Debug,
    rc::{Rc, Weak},
};

pub trait IWeakListNode: Sized {
    /// 加载链表头部信息 (prev, next)
    fn load_head(&self) -> (Weak<Self>, Weak<Self>);

    /// 存储链表头部信息 (prev, next)
    fn store_head(&self, head: (Weak<Self>, Weak<Self>));

    /// 创建一个新的哨兵节点
    fn new_sentinel() -> Rc<Self>;

    /// 检查当前节点是否为哨兵节点
    fn is_sentinel(&self) -> bool;

    /// 获取下一个节点的弱引用
    fn get_next(&self) -> Weak<Self> {
        self.load_head().1
    }

    /// 获取前一个节点的弱引用
    fn get_prev(&self) -> Weak<Self> {
        self.load_head().0
    }

    /// 设置前一个节点
    fn set_prev(&self, prev: Weak<Self>) {
        let (_, next) = self.load_head();
        self.store_head((prev, next));
    }

    /// 设置下一个节点
    fn set_next(&self, next: Weak<Self>) {
        let (prev, _) = self.load_head();
        self.store_head((prev, next));
    }

    /// 同时设置前后节点
    fn set_prev_next(&self, prev: Weak<Self>, next: Weak<Self>) {
        self.store_head((prev, next));
    }

    /// 检查节点是否已经连接到链表中
    fn is_attached(&self) -> bool {
        let null = Weak::new();
        let (prev, next) = self.load_head();
        let prev_null = prev.ptr_eq(&null);
        let next_null = next.ptr_eq(&null);
        debug_assert_eq!(prev_null, next_null, "Found broken WeakList link");
        !prev_null
    }

    /// 将节点插入到指定位置 (在 prev 和 next 之间)
    fn attach(self: &Rc<Self>, prev: Weak<Self>, next: Weak<Self>) {
        self.set_prev_next(prev.clone(), next.clone());
        prev.upgrade().map(|p| p.set_next(Rc::downgrade(self)));
        next.upgrade().map(|n| n.set_prev(Rc::downgrade(self)));
    }

    /// 将节点从链表中移除
    fn detach(&self) {
        let (prev, next) = self.load_head();
        prev.upgrade().map(|p| p.set_next(next.clone()));
        next.upgrade().map(|n| n.set_prev(prev.clone()));
        self.set_prev_next(Weak::new(), Weak::new());
    }

    /// 当链表析构时通知到该结点, 该结点应该怎么做.
    /// 该函数调用时, 结点已经从链表中移除, 但仍然存在于内存中.
    fn on_list_finalize(&self);
}

pub struct WeakList<T: IWeakListNode> {
    pub sentinal: Rc<T>,
}

impl<T: IWeakListNode + Debug> Debug for WeakList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.into_iter()).finish()
    }
}

impl<T: IWeakListNode> Drop for WeakList<T> {
    fn drop(&mut self) {
        // 在链表析构时通知所有节点
        let mut current = self.sentinal.get_next();
        let sentinal_weak = Rc::downgrade(&self.sentinal);
        while !current.ptr_eq(&sentinal_weak) {
            let Some(current_strong) = current.upgrade() else {
                panic!("Found a non-existing node in WeakList during drop");
            };
            let next = current_strong.get_next(); // 提前获取下一个节点
            // 从链表中移除当前节点
            current_strong.detach();
            // 调用节点的析构通知
            current_strong.on_list_finalize();
            current = next;
        }
    }
}

impl<T: IWeakListNode> WeakList<T> {
    pub fn new_empty() -> Self {
        let sentinal = T::new_sentinel();
        sentinal.set_prev(Rc::downgrade(&sentinal));
        sentinal.set_next(Rc::downgrade(&sentinal));
        WeakList { sentinal }
    }

    pub fn is_empty(&self) -> bool {
        self.sentinal
            .get_next()
            .ptr_eq(&Rc::downgrade(&self.sentinal))
    }
    pub fn is_single(&self) -> bool {
        let next = self.sentinal.get_next();
        let prev = self.sentinal.get_prev();
        let weak_sentinal = Rc::downgrade(&self.sentinal);
        next.ptr_eq(&prev) && !next.ptr_eq(&weak_sentinal)
    }
    pub fn is_multiple(&self) -> bool {
        let next = self.sentinal.get_next();
        let prev = self.sentinal.get_prev();
        let weak_sentinal = Rc::downgrade(&self.sentinal);
        !next.ptr_eq(&prev) && !next.ptr_eq(&weak_sentinal)
    }

    pub fn iter(&self) -> WeakListIter<T> {
        WeakListIter { current: self.sentinal.get_next() }
    }

    pub fn push_front_rc(&self, node: &Rc<T>) {
        self.push_front(Rc::downgrade(node));
    }
    pub fn push_front(&self, node: Weak<T>) {
        let Some(node_rc) = node.upgrade() else {
            panic!("Tried to push a non-existing node to WeakList");
        };
        if node_rc.is_attached() {
            panic!("Tried to push an already attached node to WeakList");
        }
        node_rc.attach(Rc::downgrade(&self.sentinal), self.sentinal.get_next());
    }

    pub fn push_back_rc(&self, node: &Rc<T>) {
        self.push_back(Rc::downgrade(node));
    }
    pub fn push_back(&self, node: Weak<T>) {
        let Some(node_rc) = node.upgrade() else {
            panic!("Tried to push a non-existing node to WeakList");
        };
        if node_rc.is_attached() {
            panic!("Tried to push an already attached node to WeakList");
        }
        node_rc.attach(self.sentinal.get_prev(), Rc::downgrade(&self.sentinal));
    }

    pub fn len(&self) -> usize {
        self.iter().count()
    }
    pub fn clear(&self) {
        let node = self.sentinal.get_next();
        let sentinal_weak = Rc::downgrade(&self.sentinal);
        let mut current = node;
        while !current.ptr_eq(&sentinal_weak) {
            if let Some(current_strong) = current.upgrade() {
                let next = current_strong.get_next();
                current_strong.detach();
                current = next;
            } else {
                break; // 遇到已释放的节点，停止清理
            }
        }
    }
    pub fn contains(&self, node: &Rc<T>) -> bool {
        self.iter().any(|n| Rc::ptr_eq(&n, node))
    }
    pub fn front_weak(&self) -> Option<Weak<T>> {
        let next = self.sentinal.get_next();
        if next.ptr_eq(&Rc::downgrade(&self.sentinal)) { None } else { Some(next) }
    }
    pub fn front(&self) -> Option<Rc<T>> {
        self.sentinal
            .get_next()
            .upgrade()
            .and_then(|n| if n.is_sentinel() { None } else { Some(n) })
    }
    pub fn back_weak(&self) -> Option<Weak<T>> {
        let prev = self.sentinal.get_prev();
        if prev.ptr_eq(&Rc::downgrade(&self.sentinal)) { None } else { Some(prev) }
    }
    pub fn back(&self) -> Option<Rc<T>> {
        self.sentinal
            .get_prev()
            .upgrade()
            .and_then(|n| if n.is_sentinel() { None } else { Some(n) })
    }

    /// 克隆一个视图, 但不克隆节点. UseList 在设计上就不支持深拷贝.
    pub fn clone_view(&self) -> Self {
        WeakList { sentinal: self.sentinal.clone() }
    }

    /// 移动所有节点到另一个列表, 并清空自己.
    pub fn move_all_to(&self, other: &WeakList<T>, mut on_move: impl FnMut(&Rc<T>)) {
        if Rc::ptr_eq(&self.sentinal, &other.sentinal) {
            return;
        }
        let self_front = self.sentinal.get_next(); // 链表头
        let self_back = self.sentinal.get_prev(); // 链表尾

        if self_front.ptr_eq(&Rc::downgrade(&self.sentinal)) {
            // 自己是空的, 没什么可搬的.
            return;
        }

        // 清空自己.
        self.sentinal.set_next(Rc::downgrade(&self.sentinal));
        self.sentinal.set_prev(Rc::downgrade(&self.sentinal));

        let other_back = other.sentinal.get_prev();
        // 把自己的链表尾接到 other 的最后面 -- `sentinal.prev`.
        other.sentinal.set_prev(self_back.clone());

        // 把自己的链表头接到 other 表尾的后面 -- `other_back.next`.
        other_back.upgrade().map(|p| p.set_next(self_front.clone()));

        // 修正被转移链表的边界连接
        self_back
            .upgrade()
            .map(|p| p.set_next(Rc::downgrade(&other.sentinal)));
        self_front.upgrade().map(|p| p.set_prev(other_back.clone()));

        // 旧链表结点的前后指针不需要修正——头尾结点已经接到 new_list 上了, 中间结点的 prev/next 仍然有效.

        // 遍历所有被转移的结点, 调用 `on_move` 处理其他操作.
        let mut current = self_front;
        let target_sentinel = Rc::downgrade(&other.sentinal);
        while !current.ptr_eq(&target_sentinel) {
            let Some(current_strong) = current.upgrade() else {
                panic!("Found a non-existing node in WeakList during move_to");
            };
            let next = current_strong.get_next(); // 提前获取下一个节点
            on_move(&current_strong);
            current = next;
        }
    }

    /// 根据条件移动节点到另一个列表, 并在移动时调用回调函数.
    /// 只移动满足 `predicate` 条件的节点. 保持节点在原列表中的相对顺序.
    /// 注意: 该操作会遍历整个列表, 复杂度为 O(n).
    pub fn move_to_if(
        &self,
        other: &WeakList<T>,
        mut predicate: impl FnMut(&Rc<T>) -> bool,
        mut on_move: impl FnMut(&Rc<T>),
    ) {
        if Rc::ptr_eq(&self.sentinal, &other.sentinal) {
            return;
        }

        let mut current = self.sentinal.get_next();
        let self_sentinel_weak = Rc::downgrade(&self.sentinal);

        while !current.ptr_eq(&self_sentinel_weak) {
            let Some(current_strong) = current.upgrade() else {
                // 遇到已释放的节点，链表损坏, 停止移动
                panic!("Found a non-existing node in WeakList during move_to_if");
            };

            let next = current_strong.get_next(); // 提前获取下一个节点
            if predicate(&current_strong) {
                // 从当前链表中移除
                current_strong.detach();
                // 动态获取目标链表的当前尾部，确保插入到最后
                other.push_back(Rc::downgrade(&current_strong));
                on_move(&current_strong);
            }
            current = next;
        }
    }
}

pub struct WeakListIter<T: IWeakListNode> {
    pub current: Weak<T>,
}

impl<T: IWeakListNode> Iterator for WeakListIter<T> {
    type Item = Rc<T>;

    fn next(&mut self) -> Option<Rc<T>> {
        let current = self.current.upgrade()?;
        if current.is_sentinel() {
            None
        } else {
            self.current = current.get_next();
            Some(current)
        }
    }
}

impl<T: IWeakListNode> IntoIterator for &WeakList<T> {
    type Item = Rc<T>;
    type IntoIter = WeakListIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: IWeakListNode> DoubleEndedIterator for WeakListIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let current = self.current.upgrade()?;
        if current.is_sentinel() {
            None
        } else {
            self.current = current.get_prev();
            Some(current)
        }
    }
}

impl<T: IWeakListNode> IntoIterator for WeakList<T> {
    type Item = Rc<T>;
    type IntoIter = WeakListIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
