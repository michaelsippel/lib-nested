#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ListCursorMode {
    Insert,
    Select
}

#[derive(Clone, Copy, Eq, PartialEq)]
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
        ListCursor::none()
    }
}

