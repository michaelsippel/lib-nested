
use {
    r3vi::view::singleton::SingletonView,
    crate::{
        repr_tree::{ReprTree, Context},
        edit_tree::{TreeNav, TreeNavResult},
        editors::{ObjCommander, digit::DigitEditor}
    },

    std::sync::{Arc, RwLock}
};

impl ObjCommander for DigitEditor {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) -> TreeNavResult {
        let cmd_obj = cmd_obj.read().unwrap();
        let cmd_type = cmd_obj.get_type().clone();

        if cmd_type == Context::parse(&self.ctx, "Char") {
            if let Some(cmd_view) = cmd_obj.get_view::<dyn SingletonView<Item = char>>() {
                let c = cmd_view.get();

                self.msg.clear();

                if self.ctx.read().unwrap().meta_chars.contains(&c) {
                    return TreeNavResult::Exit;

                } else if c.to_digit(self.radix).is_none() {
                    /* in case the character c is not in the range of digit-chars,
                       add a message to the diagnostics view
                     */
/*
                    let message = IndexBuffer::from_iter(vec![
                        (Point2::new(1, 0), make_label("invalid digit '")),
                        (Point2::new(2, 0), make_label(&format!("{}", c))
                         .map_item(|_p,a| a.add_style_back(TerminalStyle::fg_color((140,140,250))))),
                        (Point2::new(3, 0), make_label("'"))
                    ]);

                    self.msg.push(crate::diagnostics::make_error(message.get_port().flatten()));
*/

                    self.data.set(c);
                } else {
                    self.data.set(c);
                }
            }
        }

        TreeNavResult::Continue
    }
}

