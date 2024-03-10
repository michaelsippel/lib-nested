use {
    r3vi::{
        view::{
            OuterViewPort,
            singleton::*,
        },
        buffer::singleton::*
    },
    laddertypes::{TypeTerm},
    crate::{
        repr_tree::{Context, ReprTree},
        edit_tree::{EditTree, TreeNavResult},
        editors::ObjCommander,
    },
    std::sync::Arc,
    std::sync::RwLock
};

pub fn init_ctx( ctx: Arc<RwLock<Context>> ) {
    
    let morphtype =
            crate::repr_tree::MorphismType {
                src_type: Context::parse(&ctx, "Char"),
                dst_type: Context::parse(&ctx, "Char~EditTree")
            };

    ctx.write().unwrap()
        .morphisms
        .add_morphism(
            morphtype,
            {
                let ctx = ctx.clone();
                move |rt, σ| {
                    if let Some(v) = rt.read().unwrap().get_view::<dyn SingletonView<Item = char>>() {
                        eprintln!("prev value: {}", v.get());
                    }

                    /* Create EditTree object
                     */
                    let mut edittree_char = CharEditor::new_edit_tree(
                        ctx.clone(),
                        SingletonBuffer::new('>'),
                        r3vi::buffer::singleton::SingletonBuffer::<usize>::new(0).get_port()
                    );

                    /* Insert EditTree into ReprTree
                     */
                    let mut rt = rt.write().unwrap();
                    rt.insert_leaf(
                        vec![ Context::parse(&ctx, "EditTree") ].into_iter(),
                        SingletonBuffer::new( Arc::new(RwLock::new( edittree_char )) ).get_port().into()
                    );
                }
            }
        );
}

pub struct CharEditor {
    ctx: Arc<RwLock<Context>>,
    data: SingletonBuffer<char>
}

impl ObjCommander for CharEditor {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) -> TreeNavResult {
        let cmd_obj = cmd_obj.read().unwrap();
        let cmd_type = cmd_obj.get_type().clone();

        if cmd_type == Context::parse(&self.ctx, "Char") {
            if let Some(cmd_view) = cmd_obj.get_view::<dyn SingletonView<Item = char>>() {
                let value = cmd_view.get();

                if self.ctx.read().unwrap().meta_chars.contains(&value) {
                    TreeNavResult::Exit
                } else {
                    self.data.set(value);
                    TreeNavResult::Continue
                }
            } else {
                TreeNavResult::Exit
            }
        } else {
            TreeNavResult::Exit
        }
    }
}

impl CharEditor {
    pub fn new(ctx: Arc<RwLock<Context>>) -> Self {
        CharEditor {
            ctx,
            data: SingletonBuffer::new('\0')
        }
    }

    pub fn get_port(&self) -> OuterViewPort<dyn SingletonView<Item = char>> {
        self.data.get_port()
    }

    pub fn get(&self) -> char {
        self.get_port().get_view().unwrap().get()
    }

    pub fn new_edit_tree(
        ctx0: Arc<RwLock<Context>>,
        data: SingletonBuffer<char>,
        depth: OuterViewPort<dyn SingletonView<Item = usize>>
    ) -> EditTree {
        let data = SingletonBuffer::new('\0');
        let ctx = ctx0.clone();
        let editor = Arc::new(RwLock::new(CharEditor{ ctx, data: data.clone() }));

        EditTree::new(
            ctx0.clone(),
            depth
        )
            .set_cmd( editor.clone() )
            .set_editor( editor.clone() )
    }
}

