use {
    r3vi::{
        view::{OuterViewPort, singleton::*, sequence::*},
        buffer::{singleton::*, vec::*}
    },
    laddertypes::{TypeTerm},
    crate::{
        repr_tree::{Context, ReprTree},
        edit_tree::{NestedNode, TreeNav, TreeCursor, diagnostics::Diagnostics},
        editors::{list::{ListCursor, ListCursorMode, ListCmd}, ObjCommander},
    },
    std::sync::{Arc, RwLock}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct ListEditor {
    pub cursor: SingletonBuffer<ListCursor>,

    // todo: (?) remove RwLock<..> around NestedNode ??
    pub data: VecBuffer< Arc<RwLock<NestedNode>> >,

    pub spillbuf: Arc<RwLock<Vec<Arc<RwLock<NestedNode>>>>>,

    pub(super) addr_port: OuterViewPort<dyn SequenceView<Item = isize>>,
    pub(super) mode_port: OuterViewPort<dyn SingletonView<Item = ListCursorMode>>,

    depth: OuterViewPort<dyn SingletonView<Item = usize>>,

    pub ctx: Arc<RwLock<Context>>,

    /// item type
    pub typ: TypeTerm,
}

impl ListEditor {
    pub fn new(
        ctx: Arc<RwLock<Context>>,
        typ: TypeTerm,
    ) -> Self {
        let cursor = SingletonBuffer::new(ListCursor::default());
        let data : VecBuffer<Arc<RwLock<NestedNode>>> = VecBuffer::new();

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
                                    if idx >= 0 && idx < data.len() as isize {
                                        data.get(idx as usize).read().unwrap().get_mode_view()
                                    } else {
                                        eprintln!("ListEditor::mode_port invalid cursor idx");
                                        ip
                                    }
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
            spillbuf: Arc::new(RwLock::new(Vec::new())),
            ctx,
            typ,
            depth: SingletonBuffer::new(0).get_port()
        }
    }

    pub fn into_node(mut self, depth: OuterViewPort<dyn SingletonView<Item = usize>>) -> NestedNode {
        let data = self.get_data();
        let ctx = self.ctx.clone();

        self.depth = depth.clone();
        let editor = Arc::new(RwLock::new(self));

        let e = editor.read().unwrap();

        let mut node = NestedNode::new(ctx, data, depth)
            .set_editor(editor.clone())
            .set_nav(editor.clone())
            .set_cmd(editor.clone())
            .set_diag(e
                      .get_data_port()
                      .enumerate()
                      .map(|(idx, item_editor)| {
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
                          })
                      .flatten()
            );

        node.spillbuf = e.spillbuf.clone();
        node
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

    /// is the element-type also a list-like editor (i.e. impls TreeNav)
    pub fn is_listlist(&self) -> bool {
        self.ctx.read().unwrap().is_list_type(&self.typ)
    }

    /// delete all items
    pub fn clear(&mut self) {
        let mut b = self.spillbuf.write().unwrap();
        for i in 0..self.data.len() {
            b.push( self.data.get(i) );
        }
        
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
        eprintln!("list insert");

        item.read().unwrap().depth.0.set_view(
            self.depth.map(|d| d+1).get_view()
        );

        let mut cur = self.cursor.get();
        if let Some(idx) = cur.idx {
            match cur.mode {
                ListCursorMode::Insert => {
                    self.data.insert(idx as usize, item.clone());
                    if self.is_listlist() {
                        cur.mode = ListCursorMode::Select;
                    } else {
                        eprintln!("list insert: is not a listlist ({:?})", self.typ);
                        item.write().unwrap().goto(TreeCursor::none());
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
    pub fn split(&mut self) {
        eprintln!("split");
        let cur = self.cursor.get();
        if let Some(idx) = cur.idx {
            let idx = idx as usize;
            for _ in idx .. self.data.len() {
                self.spillbuf.write().unwrap().push(
                    self.data.get(idx)
                );
                self.data.remove(idx);
            }

            /* in case the split leaves an empty item-list
             * as a last element, remove it
             */
/*
            if self.is_listlist() {
                if idx > 0 && idx < self.data.len()+1 {
                    /* we are in insert mode,
                     * get element before cursor
                     */
                    let prev_idx = idx - 1;
                    let prev_node = self.data.get(prev_idx);
                    let prev_node = prev_node.read().unwrap();

                    if prev_node.get_data_view::<dyn SequenceView<Item = NestedNode>>(vec![].into_iter()).iter().count() == 0 {
                        drop(prev_node);
                        self.data.remove(prev_idx);
                    }
                }
        }
            */
        }
    }

    pub fn listlist_split(&mut self) {
        eprintln!("listlist split");
        let cur = self.get_cursor();

        if let Some(mut item) = self.get_item().clone() {
            item.send_cmd_obj(ListCmd::Split.into_repr_tree(&self.ctx));

            if cur.tree_addr.len() < 3 {
                item.goto(TreeCursor::none());

                self.set_leaf_mode(ListCursorMode::Insert);
                self.nexd();

                let mut b = item.spillbuf.write().unwrap();
                let mut tail_node = Context::make_node(&self.ctx, self.typ.clone(), self.depth.map(|d| d+1)).unwrap();
                tail_node.goto(TreeCursor::home());

                for node in b.iter() {
                    eprintln!("splid :send to tail node");
                    tail_node
                        .send_cmd_obj(
                            ReprTree::new_leaf(
                                Context::parse(&self.ctx, "NestedNode"),
                                SingletonBuffer::<NestedNode>::new(
                                    node.read().unwrap().clone()
                                ).get_port().into()
                            )
                        );
                }
                b.clear();
                drop(b);
                drop(item);

                tail_node.goto(TreeCursor::home());
                if cur.tree_addr.len() > 1 {
                    tail_node.dn();
                }

                self.insert(
                    Arc::new(RwLock::new(tail_node))
                );

            } else {
                self.up();
                self.listlist_split();
                self.dn();
            }
        }
    }

    pub fn listlist_join_pxev(&mut self, idx: isize) {
        {
            let cur_editor = self.data.get(idx as usize);
            let pxv_editor = self.data.get(idx as usize-1);
            let mut cur_editor = cur_editor.write().unwrap();
            let mut pxv_editor = pxv_editor.write().unwrap();

            let oc0 = cur_editor.get_cursor();

            // tell cur_editor move all its elements into its spill-buffer
            cur_editor.goto(TreeCursor::none());
            cur_editor.send_cmd_obj(
                ListCmd::Clear.into_repr_tree( &self.ctx )
            );
            
            pxv_editor.goto(TreeCursor {
                tree_addr: vec![-1],
                leaf_mode: ListCursorMode::Insert
            });

            let old_cur = pxv_editor.get_cursor();

            let data = cur_editor.spillbuf.read().unwrap();
            for x in data.iter() {
                pxv_editor.send_cmd_obj(
                    ReprTree::new_leaf(
                        Context::parse(&self.ctx, "NestedNode"),
                        SingletonBuffer::<NestedNode>::new(
                            x.read().unwrap().clone()
                        ).get_port().into()
                    )
                );
            }


            // fixme: is it oc0 or old_cur ??
            if oc0.tree_addr.len() > 1 {
                pxv_editor.goto(TreeCursor {
                    tree_addr: vec![ old_cur.tree_addr[0], 0 ],
                    leaf_mode: ListCursorMode::Insert                
                });
                pxv_editor.send_cmd_obj(ListCmd::DeletePxev.into_repr_tree( &self.ctx ));
            } else if oc0.tree_addr.len() > 0 {
                pxv_editor.goto(TreeCursor {
                    tree_addr: vec![ old_cur.tree_addr[0] ],
                    leaf_mode: ListCursorMode::Insert                
                });
            }
        }

        self.cursor.set(ListCursor {
            idx: Some(idx as isize - 1),
            mode: ListCursorMode::Select
        });

        // remove cur_editor from top list, its elements are now in pxv_editor
        self.data.remove(idx as usize);
    }

    pub fn listlist_join_nexd(&mut self, idx: usize) {
        {
            let cur_editor = self.data.get(idx);
            let nxd_editor = self.data.get(idx + 1);
            let mut cur_editor = cur_editor.write().unwrap();
            let mut nxd_editor = nxd_editor.write().unwrap();

            let oc0 = cur_editor.get_cursor();

            // tell next_editor move all its elements into its spill-buffer
            nxd_editor.goto(TreeCursor::none());
            nxd_editor.send_cmd_obj(
                ListCmd::Clear.into_repr_tree( &self.ctx )
            );

            let old_cur = cur_editor.get_cursor();
            cur_editor.goto(TreeCursor {
                tree_addr: vec![-1],
                leaf_mode: ListCursorMode::Insert
            });
 
            let data = nxd_editor.spillbuf.read().unwrap();

            for x in data.iter() {
                cur_editor.send_cmd_obj(
                    ReprTree::new_leaf(
                        Context::parse(&self.ctx, "NestedNode"),
                        SingletonBuffer::<NestedNode>::new(
                            x.read().unwrap().clone()
                        ).get_port().into()
                    )
                );
            }

            // fixme: is it oc0 or old_cur ??
            if oc0.tree_addr.len() > 1 {
                cur_editor.goto(TreeCursor {
                    tree_addr: vec![ old_cur.tree_addr[0], -1 ],
                    leaf_mode: ListCursorMode::Insert                
                });
                cur_editor.send_cmd_obj(ListCmd::DeleteNexd.into_repr_tree( &self.ctx ));
            } else if oc0.tree_addr.len() > 0 {
                cur_editor.goto(TreeCursor {
                    tree_addr: vec![ old_cur.tree_addr[0] ],
                    leaf_mode: ListCursorMode::Insert
                });
            } else {
                cur_editor.goto(TreeCursor::none());
            }
        }

        // remove next_editor from top list, its elements are now in cur_editor
        self.data.remove(idx+1);
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


