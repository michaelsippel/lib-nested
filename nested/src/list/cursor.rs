
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ListCursorMode {
    Insert,
    Select,
    Modify
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ListCursor {
    pub mode: ListCursorMode,
    pub idx: usize
}

impl Default for ListCursor {
    fn default() -> Self {
        ListCursor {
            mode: ListCursorMode::Insert,
            idx: 0
        }
    }
}

/*
pub trait ListNav {
    fn pxev(&mut self) -> ListNavResult;
    fn nexd(&mut self) -> ListNavResult;
    fn pua(&mut self) -> ListNavResult;
    fn end(&mut self) -> ListNavResult;

    fn set_cursor(&mut self, new_cursor: Option<ListCursor>) -> ListNavResult;
    fn get_cursor(&self) -> Option<ListCursor>;
}
*/

