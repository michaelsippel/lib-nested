use {
    r3vi::{
        view::{
            OuterViewPort,
            singleton::*,
        },
        buffer::{
            singleton::*,
            vec::*,
            index_hashmap::*
        }
    },
    laddertypes::{TypeTerm},
    crate::{
        editors::{
            digit::DigitEditor,
            list::{ListCmd},
            ObjCommander
        },
        repr_tree::{Context, ReprTree},
        edit_tree::{EditTree, TreeNav, TreeNavResult, TreeCursor, diagnostics::{Message}},
    },
    std::sync::Arc,
    std::sync::RwLock,
    std::iter::FromIterator,
    cgmath::{Point2}
};

pub struct PosIntEditor {
    radix: u32,
    digits: EditTree,

    // todo: endianness
}

impl PosIntEditor {
    pub fn new(ctx: Arc<RwLock<Context>>, radix: u32) -> Self {
        PosIntEditor {
            radix,
            digits: EditTree::new(
                ctx,
                r3vi::buffer::singleton::SingletonBuffer::new(0).get_port()
            )
        }
    }

    pub fn from_u64(ctx: Arc<RwLock<Context>>, radix: u32, value: u64) -> Self {
        let mut edit = PosIntEditor::new(ctx, radix);
        edit.set_value_u64( value );
        edit
    }

    pub fn set_value_u64(&mut self, mut value: u64) {
        self.digits.send_cmd_obj(ListCmd::Clear.into_repr_tree(&self.digits.ctx));

        while value > 0 {
            let digit_val = (value % self.radix as u64) as u32;
            value /= self.radix as u64;

            // if BigEndian
            self.digits.goto(TreeCursor::home());

            self.digits.send_cmd_obj(ReprTree::from_char(&self.digits.ctx, char::from_digit(digit_val, self.radix).expect("invalid digit"))); 
        }
        self.digits.goto(TreeCursor::none());
    }

    pub fn into_node(self) -> EditTree {
        self.digits
    }

/*
    pub fn get_data_port(&self) -> OuterViewPort<dyn SequenceView<Item = u32>> {
        let radix = self.radix;
        self.digits
            .get_data_port()
            .filter_map(move |digit_editor| {
                digit_editor.read().unwrap().data.get()?.to_digit(radix)
            })
    }

    pub fn get_value(&self) -> u32 {
        let mut value = 0;
        let mut weight = 1;
        for digit_value in self
            .get_data_port()
            .get_view()
            .unwrap()
            .iter()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            value += digit_value * weight;
            weight *= self.radix;
        }

        value
}
*/
}

