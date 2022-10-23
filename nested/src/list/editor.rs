use {
    crate::{
        core::{OuterViewPort, ViewPort},
        list::{
            ListCursor, ListCursorMode,
            ListSegment, ListSegmentSequence,
            segment::PTYSegment
        },
        sequence::{SequenceView},
        singleton::{SingletonBuffer, SingletonView},
        terminal::{
            make_label, TerminalEditor, TerminalEditorResult, TerminalEvent, TerminalStyle,
            TerminalView,
        },
        tree_nav::{TerminalTreeEditor, TreeCursor, TreeNav, TreeNavResult},
        vec::VecBuffer,
        color::{bg_style_from_depth, fg_style_from_depth}
    },
    std::sync::{Arc, RwLock},
    termion::event::{Event, Key},
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                                                            
pub struct ListEditor<ItemEditor>
where ItemEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static
{
    pub(super) cursor: SingletonBuffer<ListCursor>,
    pub(super) data: VecBuffer<Arc<RwLock<ItemEditor>>>,
    pub(super) make_item_editor: Box<dyn Fn() -> Arc<RwLock<ItemEditor>> + Send + Sync>,

    pub(super) depth: usize,
    pub(super) cur_dist: Arc<RwLock<usize>>,
}

impl<ItemEditor> ListEditor<ItemEditor>
where ItemEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static
{
    pub fn new(make_item_editor: impl Fn() -> Arc<RwLock<ItemEditor>> + Send + Sync + 'static, depth: usize) -> Self {
        ListEditor {
            cursor: SingletonBuffer::new(ListCursor::default()),
            data: VecBuffer::<Arc<RwLock<ItemEditor>>>::new(),
            make_item_editor: Box::new(make_item_editor),
            depth,
            cur_dist: Arc::new(RwLock::new(0)),
        }
    }

    pub fn get_seg_seq_view(
        &self,
    ) -> OuterViewPort<dyn SequenceView<Item = OuterViewPort<dyn TerminalView>>> {
        let segment_view_port = ViewPort::<dyn SequenceView<Item = ListSegment<ItemEditor>>>::new();
        ListSegmentSequence::new(
            self.get_cursor_port(),
            self.get_data_port(),
            segment_view_port.inner(),
            self.depth
        );
        segment_view_port.into_outer().map(move |segment| segment.pty_view())
    }
    
    pub fn get_data_port(&self) -> OuterViewPort<dyn SequenceView<Item = Arc<RwLock<ItemEditor>>>> {
        self.data.get_port().to_sequence()
    }

    pub fn get_cursor_port(&self) -> OuterViewPort<dyn SingletonView<Item = ListCursor>> {
        self.cursor.get_port()
    }

    pub fn get_item(&self) -> Option<Arc<RwLock<ItemEditor>>> {
        if let Some(idx) = self.cursor.get().idx {
            let idx = crate::modulo(idx as isize, self.data.len() as isize) as usize;
            if idx < self.data.len() {
                Some(self.data.get(idx))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// split the list off at the current cursor position and return the second half
    /*
    pub fn split(&mut self) -> ListEditor<ItemEditor> {
        let mut le = ListEditor::new(self.make_item_editor.clone());
        let p = self.cursor.get();
        for i in p.idx .. self.data.len() {
            le.data.push( self.data[p.idx] );
            self.data.remove(p.idx);
        }
        le.goto(TreeCursor::home());
        le
    }
     */

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

