struct DSUNode {
    parent: usize,
}

pub struct DSU {
    nodes: Vec<DSUNode>,
}

impl DSU {
    pub fn new(size: usize) -> Self {
        let mut nodes = Vec::with_capacity(size);
        for i in 0..size {
            nodes.push(DSUNode { parent: i });
        }
        Self { nodes }
    }

    pub fn find(&mut self, x: usize) -> usize {
        if self.nodes[x].parent != x {
            self.nodes[x].parent = self.find(self.nodes[x].parent);
        }
        self.nodes[x].parent
    }
    pub fn find_when<T>(&mut self, x: usize, mut on_update: T) -> usize
    where
        T: FnMut(/* x: */ usize, /* old parent */ usize, /* new parent */ usize),
    {
        if self.nodes[x].parent == x {
            return x;
        }
        let old_parent = self.nodes[x].parent;
        self.nodes[x].parent = self.find(self.nodes[x].parent);
        let new_parent = self.nodes[x].parent;
        on_update(x, old_parent, new_parent);
        new_parent
    }

    pub fn readonly_find(&self, x: usize) -> usize {
        if self.nodes[x].parent != x {
            return self.readonly_find(self.nodes[x].parent);
        }
        self.nodes[x].parent
    }
    pub fn get_direct_parent(&self, x: usize) -> usize {
        self.nodes[x].parent
    }
    pub fn set_direct_parent(&mut self, x: usize, parent: usize) {
        assert!(parent < self.nodes.len());
        self.nodes[x].parent = parent;
    }

    pub fn is_connected(&mut self, x: usize, y: usize) -> bool {
        self.find(x) == self.find(y)
    }
    pub fn readonly_is_connected(&self, x: usize, y: usize) -> bool {
        self.readonly_find(x) == self.readonly_find(y)
    }

    pub fn union(&mut self, mut x: usize, mut y: usize) {
        x = self.find(x);
        y = self.find(y);

        if x != y {
            self.nodes[x].parent = y;
        }
    }
}
