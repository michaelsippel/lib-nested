use {
    r3vi::{
        view::{port::UpdateTask, OuterViewPort, singleton::*, sequence::*},
        buffer::{singleton::*, vec::*}
    },
    crate::{
        type_system::{Context, TypeTerm, ReprTree},
        editors::list::{ListCursor, ListCursorMode},
        tree::{NestedNode, TreeNav, TreeCursor},
        diagnostics::Diagnostics
    },
    std::sync::{Arc, RwLock}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct ListEditor {
    pub(super) cursor: SingletonBuffer<ListCursor>,
    pub data: VecBuffer< Arc<RwLock<NestedNode>> >,

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
        let data = VecBuffer::<Arc<RwLock<NestedNode>>>::new();

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
                                    data.get(idx as usize).read().unwrap().get_mode_view()
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
                                            return data.get(idx as usize).read().unwrap().get_addr_view();
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

    pub fn into_node(self, depth: usize) -> NestedNode {
        let data = self.get_data();
        let ctx = self.ctx.clone();
        let editor = Arc::new(RwLock::new(self));

        let e = editor.read().unwrap();

        NestedNode::new(ctx, data, depth)
            .set_editor(editor.clone())
            .set_nav(editor.clone())
            .set_cmd(editor.clone())
            .set_diag(e
                      .get_data_port()
                      .enumerate()
                      .map(
                          |(idx, item_editor)| {
                              let idx = *idx;
                              item_editor
                                  .get_msg_port()
                                  .map(
                                      move |msg| {
                                          let mut msg = msg.clone();
                                          msg.addr.insert(0, idx);
                                          msg
                                      }
                                  )
                          }
                      )
                      .flatten()
            )
    }

    pub fn get_item_type(&self) -> TypeTerm {
        self.typ.clone()
    }

    pub fn get_seq_type(&self) -> TypeTerm {
        TypeTerm::App(vec![
            TypeTerm::TypeID(self.ctx.read().unwrap().get_typeid("List").unwrap()),
            self.get_item_type().into()
        ])
    }

    pub fn get_cursor_port(&self) -> OuterViewPort<dyn SingletonView<Item = ListCursor>> {
        self.cursor.get_port()
    }

    pub fn get_data_port(&self) -> OuterViewPort<dyn SequenceView<Item = NestedNode>> {
        self.data.get_port().to_sequence().map(
            |x| x.read().unwrap().clone()
        )
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
                Some(self.data.get(idx).read().unwrap().clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_item_mut(&mut self) -> Option<MutableVecAccess<Arc<RwLock<NestedNode>>>> {
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
    pub fn insert(&mut self, item: Arc<RwLock<NestedNode>>) {
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
        } else {
            //eprintln!("insert: no cursor");
        }
    }

    /// split the list off at the current cursor position and return the second half
    pub fn split(&mut self, le_node: &mut NestedNode) {
        let cur = self.cursor.get();
        if let Some(idx) = cur.idx {
            let idx = idx as usize;
            le_node.goto(TreeCursor::home());
            for _ in idx .. self.data.len() {

                eprintln!("send items to new tail");
                le_node.cmd.get().unwrap().write().unwrap().send_cmd_obj(
                    self.data.get(idx).read().unwrap().data.clone()
/*
                    ReprTree::new_leaf(
                        self.ctx.read().unwrap().type_term_from_str("( NestedNode )").unwrap(),
                        SingletonBuffer::<NestedNode>::new( self.data.get(idx).clone().read().unwrap().clone() ).get_port().into()
                    )
*/
                );
                self.data.remove(idx);
            }
            le_node.goto(TreeCursor::none());

            if self.is_listlist() {
                if idx > 0 && idx < self.data.len()+1 {

                    let prev_idx = idx - 1; // get last element before cursor (we are in insert mode)
                    let prev_node = self.data.get(prev_idx);
                    let prev_node = prev_node.read().unwrap();

                    if let Some(prev_editor) = prev_node.editor.get() {
                        let prev_editor = prev_editor.downcast::<RwLock<ListEditor>>().unwrap();
                        let prev_editor = prev_editor.write().unwrap();
                        prev_editor.get_data_port().0.update();

                        if prev_editor.get_data_port().get_view().unwrap().iter()
                            .filter_map(|x| x.get_data_view::<dyn SingletonView<Item = Option<char>>>(vec![].into_iter())?.get()).count() == 0
                        {
                            drop(prev_editor);
                            self.data.remove(prev_idx);
                        }
                    }
                }
            }
        }
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

    pub fn listlist_split(&mut self) {
        let cur = self.get_cursor();

        if let Some(item) = self.get_item() {
//            let item = item.read().unwrap();
            let _depth = item.depth;
            
            if let Some(head_editor) = item.editor.get() {
                eprintln!("listlistsplit:editor = {:?}", Arc::into_raw(head_editor.clone()));

                let head = head_editor.downcast::<RwLock<ListEditor>>().unwrap();
                let mut head = head.write().unwrap();

                let mut tail_node = Context::make_node(&self.ctx, self.typ.clone(), 0).unwrap();

                if head.data.len() > 0 {
                    if cur.tree_addr.len() > 2 {
                        eprintln!("call child head listlist split");
                        head.listlist_split();
                        eprintln!("return");
                    }

                    /*
                    TODO: replace this by: (does not require  ListEditor downcast)
                    head.send_cmd_obj(ListCmd::Split.into_repr());
                    tail_node = head.spill_buf.clone();
                     */

                    head.split( &mut tail_node );
                }

                head.goto(TreeCursor::none());
                drop(head);

                tail_node.goto(
                    TreeCursor {
                        tree_addr: vec![0],
                        leaf_mode: if cur.tree_addr.len() > 2 {
                            ListCursorMode::Select
                        } else {
                            ListCursorMode::Insert
                        }
                    }
                );
                self.insert(
                    Arc::new(RwLock::new(tail_node))
                );

                eprintln!("made insert");
            }
        }
    }

    pub fn listlist_join_pxev(&mut self, idx: isize, item: &NestedNode) {
        {
            let prev_editor = self.data.get_mut(idx as usize-1);
            let prev_editor = prev_editor.read().unwrap();

            if let Some(prev_editor) = prev_editor.editor.get() {
                if let Ok(prev_editor) = prev_editor.downcast::<RwLock<ListEditor>>() {
                    let mut prev_editor = prev_editor.write().unwrap();

                    let cur_editor = item.editor.get().unwrap();
                    let cur_editor = cur_editor.downcast::<RwLock<ListEditor>>().unwrap();
                    let cur_editor = cur_editor.write().unwrap();

                    prev_editor.join(&cur_editor);

                    self.cursor.set(
                        ListCursor {
                            idx: Some(idx - 1), mode: ListCursorMode::Select
                        }
                    );
                }
            }
        }

        self.data.remove(idx as usize);
    }

    pub fn listlist_join_nexd(&mut self, next_idx: usize, item: &NestedNode) {
        {
            let next_editor = self.data.get(next_idx);
            let next_editor = next_editor.read().unwrap();
            if let Some(next_editor) = next_editor.editor.get() {
                if let Ok(next_editor) = next_editor.downcast::<RwLock<ListEditor>>() {
                    let next_editor = next_editor.write().unwrap();
                    let cur_editor = item.editor.get().unwrap();
                    let cur_editor = cur_editor.downcast::<RwLock<ListEditor>>().unwrap();
                    let mut cur_editor = cur_editor.write().unwrap();

                    cur_editor.join(&next_editor);
                }
            }
        }
        self.data.remove(next_idx);
    }
}
/*
use crate::{
    type_system::TypeLadder,
    tree::{TreeType, TreeAddr}
};

impl TreeType for ListEditor {
    fn get_type(&self, addr: &TreeAddr) -> TypeLadder {
        let idx = crate::utils::modulo::modulo(addr.0[0] as isize, self.data.len() as isize) as usize;

        let mut addr = addr.clone();
        
        if self.data.len() > 0 {
            addr.0.remove(0);
            self.data.get(idx).get_type(addr)
        } else {
            vec![]
        }
    }
}
*/
