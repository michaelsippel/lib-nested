use {
    r3vi::{
        view::{
            OuterViewPort,
            singleton::*,
            sequence::*,
        },
        buffer::{
            singleton::*,
            vec::*,
        }
    },
    crate::{
        type_system::{Context, TypeTerm, ReprTree},
        editors::list::{
            ListCursor,
            ListCursorMode
        },
        tree::{NestedNode, TreeNav}
    },
    std::sync::{Arc, RwLock},
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct ListEditor {
    pub(super) cursor: SingletonBuffer<ListCursor>,
    pub(crate) data: VecBuffer<NestedNode>,

    pub(super) addr_port: OuterViewPort<dyn SequenceView<Item = isize>>,
    pub(super) mode_port: OuterViewPort<dyn SingletonView<Item = ListCursorMode>>,

    pub(crate) ctx: Arc<RwLock<Context>>,

    /// item type
    pub(super) typ: TypeTerm,
}

impl ListEditor {
    pub fn new(
        ctx: Arc<RwLock<Context>>,
        typ: TypeTerm,
    ) -> Self {
        let cursor = SingletonBuffer::new(ListCursor::default());
        let data = VecBuffer::<NestedNode>::new();

        ListEditor {
            mode_port: cursor
                .get_port()
                .map({
                    let data = data.clone();
                    move |c| {
                        let ip = SingletonBuffer::new(c.mode).get_port();
                        match c.mode {
                            ListCursorMode::Insert => ip,
                            ListCursorMode::Select => {
                                if let Some(idx) = c.idx {
                                    data.get(idx as usize).get_mode_view()
                                } else {
                                    ip
                                }
                            }
                        }
                    }
                })
                .flatten(),

            addr_port: VecBuffer::<OuterViewPort<dyn SequenceView<Item = isize>>>::with_data(
                vec![
                    cursor.get_port()
                        .to_sequence()
                        .filter_map(|cur| cur.idx),
                    cursor.get_port()
                        .map({
                            let data = data.clone();
                            move |cur| {
                                if cur.mode == ListCursorMode::Select {
                                    if let Some(idx) = cur.idx {
                                        if idx >= 0 && idx < data.len() as isize {
                                            return data.get(idx as usize).get_addr_view();
                                        }
                                    }
                                }
                                OuterViewPort::default()
                            }
                        })
                        .to_sequence()
                        .flatten()                
                ])
                .get_port()
                .to_sequence()
                .flatten(),
            cursor,
            data,
            ctx,
            typ,
        }
    }

    pub fn get_item_type(&self) -> TypeTerm {
        self.typ.clone()
    }

    pub fn get_seq_type(&self) -> TypeTerm {
        TypeTerm::Type {
            id: self.ctx.read().unwrap().get_typeid("Sequence").unwrap(),
            args: vec![ self.get_item_type() ]
        }        
    }

    pub fn into_node(self) -> NestedNode {
        let data = self.get_data();
        let editor = Arc::new(RwLock::new(self));

        NestedNode::new()
            .set_data(data)
            .set_editor(editor.clone())
            .set_nav(editor.clone())
//            .set_cmd(editor.clone())
    }

    pub fn get_cursor_port(&self) -> OuterViewPort<dyn SingletonView<Item = ListCursor>> {
        self.cursor.get_port()
    }

    pub fn get_data_port(&self) -> OuterViewPort<dyn SequenceView<Item = NestedNode>> {
        self.data.get_port().to_sequence()
    }

    pub fn get_data(&self) -> Arc<RwLock<ReprTree>> {
        let data_view = self.get_data_port();
        ReprTree::new_leaf(
            self.get_seq_type(),
            data_view.into()
        )
    }

    pub fn get_item(&self) -> Option<NestedNode> {
        if let Some(idx) = self.cursor.get().idx {
            let idx = crate::utils::modulo(idx as isize, self.data.len() as isize) as usize;
            if idx < self.data.len() {
                Some(self.data.get(idx))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_item_mut(&mut self) -> Option<MutableVecAccess<NestedNode>> {
        if let Some(idx) = self.cursor.get().idx {
            let idx = crate::utils::modulo(idx as isize, self.data.len() as isize) as usize;
            if idx < self.data.len() {
                Some(self.data.get_mut(idx))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn is_listlist(&self) -> bool {
        self.ctx.read().unwrap().is_list_type(&self.typ)
        /*
        match self.typ.clone() {
            TypeTerm::Type { id, args } => {
                id == self.ctx.read().unwrap().get_typeid("List").unwrap()
            },
            TypeTerm::Num(_) => false
        }
        */
    }

    /// delete all items
    pub fn clear(&mut self) {
        self.data.clear();
        self.cursor.set(ListCursor::home());
    }

    /// delete item before the cursor
    pub fn delete_pxev(&mut self) {
        let mut cur = self.cursor.get();
        if let Some(idx) = cur.idx {
            if idx > 0 && idx <= self.data.len() as isize {
                cur.idx = Some(idx as isize - 1);
                self.cursor.set(cur);
                self.data.remove(idx as usize - 1);
            }
        }
    }

    /// delete item after the cursor
    pub fn delete_nexd(&mut self) {
        if let Some(idx) = self.cursor.get().idx {
            if idx < self.data.len() as isize {
                self.data.remove(idx as usize);
            }
        }
    }

    /// insert a new element
    pub fn insert(&mut self, item: NestedNode) {
        let mut cur = self.cursor.get();
        if let Some(idx) = cur.idx {

            match cur.mode {
                ListCursorMode::Insert => {
                    self.data.insert(idx as usize, item.clone());
                    if self.is_listlist() {
                        cur.mode = ListCursorMode::Select;
                    } else {
                        cur.idx = Some(idx + 1);                        
                    }
                }

                ListCursorMode::Select => {
                    self.data.insert(1 + idx as usize, item.clone());                    
                    if self.is_listlist() {
                        cur.idx = Some(idx + 1);
                    }
                }
            }

            self.cursor.set(cur);
        }
    }
    
    /// split the list off at the current cursor position and return the second half
    pub fn split(&mut self) -> ListEditor {
        let mut le = ListEditor::new(self.ctx.clone(), self.typ.clone());

        let cur = self.cursor.get();
        if let Some(idx) = cur.idx {
            let idx = idx as usize;
            for _ in idx .. self.data.len() {
                le.data.push( self.data.get(idx) );
                self.data.remove(idx);
            }
        }

        le
    }

    /// append data of other editor at the end and set cursor accordingly
    pub fn join(&mut self, other: &ListEditor) {
        let selfcur = self.cursor.get();
        let othercur = other.cursor.get();

        let is_bottom = self.get_cursor().tree_addr.len() == 1 ||
            other.get_cursor().tree_addr.len() == 1;

        let is_insert =
            selfcur.mode == ListCursorMode::Insert
            || othercur.mode == ListCursorMode::Insert;

        let is_primary = self.get_cursor().tree_addr.len() > 1;

        self.cursor.set(ListCursor {
            mode: if is_insert && is_bottom {
                      ListCursorMode::Insert
                  } else {
                      ListCursorMode::Select
                  },
            idx: Some(self.data.len() as isize -
                      if is_primary {
                          1
                      } else {
                          0
                      }
            )
        });

        for i in 0 .. other.data.len() {
            self.data.push(other.data.get(i));
        }
    }
}

