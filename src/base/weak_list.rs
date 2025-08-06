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
}

pub struct WeakList<T: IWeakListNode> {
    pub sential: Rc<T>,
}

impl<T: IWeakListNode + Debug> Debug for WeakList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.into_iter()).finish()
    }
}

impl<T: IWeakListNode> WeakList<T> {
    pub fn new_empty() -> Self {
        let sential = T::new_sentinel();
        sential.set_prev(Rc::downgrade(&sential));
        sential.set_next(Rc::downgrade(&sential));
        WeakList { sential }
    }

    pub fn is_empty(&self) -> bool {
        self.sential
            .get_next()
            .ptr_eq(&Rc::downgrade(&self.sential))
    }
    pub fn is_single(&self) -> bool {
        let next = self.sential.get_next();
        let prev = self.sential.get_prev();
        let weak_sential = Rc::downgrade(&self.sential);
        next.ptr_eq(&prev) && !next.ptr_eq(&weak_sential)
    }

    pub fn iter(&self) -> WeakListIter<T> {
        WeakListIter { current: self.sential.get_next() }
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
        node_rc.attach(Rc::downgrade(&self.sential), self.sential.get_next());
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
        node_rc.attach(self.sential.get_prev(), Rc::downgrade(&self.sential));
    }

    pub fn len(&self) -> usize {
        self.iter().count()
    }
    pub fn clear(&self) {
        let node = self.sential.get_next();
        let sential_weak = Rc::downgrade(&self.sential);
        let mut current = node;
        while !current.ptr_eq(&sential_weak) {
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
        let next = self.sential.get_next();
        if next.ptr_eq(&Rc::downgrade(&self.sential)) { None } else { Some(next) }
    }
    pub fn front(&self) -> Option<Rc<T>> {
        self.sential
            .get_next()
            .upgrade()
            .and_then(|n| if n.is_sentinel() { None } else { Some(n) })
    }
    pub fn back_weak(&self) -> Option<Weak<T>> {
        let prev = self.sential.get_prev();
        if prev.ptr_eq(&Rc::downgrade(&self.sential)) { None } else { Some(prev) }
    }
    pub fn back(&self) -> Option<Rc<T>> {
        self.sential
            .get_prev()
            .upgrade()
            .and_then(|n| if n.is_sentinel() { None } else { Some(n) })
    }

    /// 克隆一个视图, 但不克隆节点. UseList 在设计上就不支持深拷贝.
    pub fn clone_view(&self) -> Self {
        WeakList { sential: self.sential.clone() }
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
