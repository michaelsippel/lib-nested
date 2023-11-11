use {
    crate::editors::list::ListCursorMode
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TreeCursor {
    pub leaf_mode: ListCursorMode,
    pub tree_addr: Vec<isize>
}

impl TreeCursor {
    pub fn home() -> Self {
        TreeCursor {
            leaf_mode: ListCursorMode::Insert,
            tree_addr: vec![0],
        }
    }

    pub fn none() -> Self {
        TreeCursor {
            leaf_mode: ListCursorMode::Select,
            tree_addr: vec![],
        }
    }

    pub fn get_subcursor(&self, depth: usize) -> TreeCursor {
        TreeCursor {
            leaf_mode: self.leaf_mode,
            tree_addr: if depth < self.tree_addr.len() {
                self.tree_addr[ depth.. ].iter().cloned().collect()
            } else {
                vec![]
            }
        }
    }
}

impl Default for TreeCursor {
    fn default() -> Self {
        TreeCursor::none()
    }
}

