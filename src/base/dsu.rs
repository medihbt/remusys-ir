struct DSUNode {
    parent: usize,
    rank: usize,
}

pub struct DSU {
    nodes: Vec<DSUNode>,
}

impl DSU {
    pub fn new(size: usize) -> Self {
        let mut nodes = Vec::with_capacity(size);
        for i in 0..size {
            nodes.push(DSUNode { parent: i, rank: 0 });
        }
        Self { nodes }
    }

    pub fn find(&mut self, x: usize) -> usize {
        if self.nodes[x].parent != x {
            self.nodes[x].parent = self.find(self.nodes[x].parent);
        }
        self.nodes[x].parent
    }
    pub fn readonly_find(&self, x: usize) -> usize {
        if self.nodes[x].parent != x {
            return self.readonly_find(self.nodes[x].parent);
        }
        self.nodes[x].parent
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

        if x == y {
            return;
        }

        if self.nodes[x].rank < self.nodes[y].rank {
            std::mem::swap(&mut x, &mut y);
        }

        self.nodes[y].parent = x;

        if self.nodes[x].rank == self.nodes[y].rank {
            self.nodes[x].rank += 1;
        }
    }
}
