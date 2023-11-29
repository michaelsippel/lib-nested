use {
    termion::event::{Event, Key},
    r3vi::{
        buffer::singleton::*
    },
    nested::{
        repr_tree::{Context, ReprTree},
        editors::list::ListCmd,
        edit_tree::nav::TreeNavCmd
    },
    crate::{
        TerminalEvent
    },
    std::sync::{Arc, RwLock}
};

fn neo2_treenav_keymap( key: &Key ) -> Option<TreeNavCmd> {
    match key {
        Key::Ctrl(c) => {
            match c {

                // left hand
                'l' => Some(TreeNavCmd::up),
                'i' => Some(TreeNavCmd::qnexd),
                'a' => Some(TreeNavCmd::dn),
                'e' => Some(TreeNavCmd::pxev),

                // right hand
                'n' => Some(TreeNavCmd::nexd),
                'r' => Some(TreeNavCmd::dn_pxev),
                't' => Some(TreeNavCmd::qnexd),
                'g' => Some(TreeNavCmd::up_nexd),

                _ => None
            }
        }
        _ => None
    }
}

fn universal_treenav_keymap( key: &Key ) -> Option<TreeNavCmd> {
    match key {    
        Key::Left => Some(TreeNavCmd::pxev),
        Key::Right => Some(TreeNavCmd::nexd),
        Key::Up => Some(TreeNavCmd::up),
        Key::Down => Some(TreeNavCmd::dn),
        Key::Home => Some(TreeNavCmd::qpxev),
        Key::End => Some(TreeNavCmd::qnexd),
        Key::PageUp => Some(TreeNavCmd::up_nexd),
        Key::PageDown => Some(TreeNavCmd::pxev_dn_qnexd),
        _ => None
    }
}

fn tty_list_keymap( key: &Key ) -> Option<ListCmd> {
    match key {
//      Key::Char('\t') => Some( ListCmd::ToggleLeafMode ),

        Key::Backspace => Some( ListCmd::DeletePxev ),
        Key::Delete => Some( ListCmd::DeleteNexd ),

        _ => None
    }
}

impl TerminalEvent {
    pub fn to_repr_tree( &self, ctx: &Arc<RwLock<Context>> ) -> Arc<RwLock<ReprTree>> {
        match self {
            TerminalEvent::Input(Event::Key(key)) => {
                if let Some(tree_nav_cmd) = neo2_treenav_keymap(key) {
                    ReprTree::new_leaf(
                        Context::parse(&ctx, "TreeNavCmd"),
                        SingletonBuffer::new(tree_nav_cmd).get_port().into()
                    )
                } else if let Some(tree_nav_cmd) = universal_treenav_keymap(key) {
                    ReprTree::new_leaf(
                        Context::parse(&ctx, "TreeNavCmd"),
                        SingletonBuffer::new(tree_nav_cmd).get_port().into()
                    )
                } else {
                    if let Some(list_cmd) = tty_list_keymap(key) {
                        ReprTree::new_leaf(
                            Context::parse(&ctx, "ListCmd"),
                            SingletonBuffer::new(list_cmd).get_port().into()
                        )
                    } else {
                        match key {
                            Key::Char(c) => {
                                ReprTree::from_char(&ctx, *c)
                            }
                            _ => {
                                ReprTree::new_leaf(
                                    Context::parse(&ctx, "TerminalEvent"),
                                    SingletonBuffer::new(self.clone()).get_port().into()
                                )
                            }
                        }
                    }
                }
            }
            _ => {
                                ReprTree::new_leaf(
                                    Context::parse(&ctx, "TerminalEvent"),
                                    SingletonBuffer::new(self.clone()).get_port().into()
                                )
            }
        }
    }
}
