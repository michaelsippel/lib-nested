#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum ListCursorMode {
    Insert,
    Select
}

impl Default for ListCursorMode {
    fn default() -> Self {
        ListCursorMode::Select
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct ListCursor {
    pub mode: ListCursorMode,
    pub idx: Option<isize>,
}

impl ListCursor {
    pub fn home() -> Self {
        ListCursor {
            mode: ListCursorMode::Insert,
            idx: Some(0)
        }
    }

    pub fn none() -> Self {
        ListCursor {
            mode: ListCursorMode::Insert,
            idx: None,
        }        
    }
}

impl Default for ListCursor {
    fn default() -> Self {
        ListCursor::home()
    }
}

