use {
    r3vi::{
        view::{OuterViewPort, sequence::*},
        buffer::{vec::*, index_hashmap::*}
    },
    crate::{
        terminal::{
            TerminalView, TerminalStyle, make_label
        }
    },
    cgmath::Point2
};

#[derive(Clone)]
pub struct Message {
    pub addr: Vec<usize>,
    pub port: OuterViewPort<dyn TerminalView>
}

pub trait Diagnostics {
    fn get_msg_port(&self) -> OuterViewPort<dyn SequenceView<Item = Message>> {
        VecBuffer::new().get_port().to_sequence()
    }
}

pub fn make_error(msg: OuterViewPort<dyn TerminalView>) -> Message {
    let mut mb = IndexBuffer::new();
    mb.insert_iter(vec![
        (Point2::new(0, 0),
         make_label("error: ")
         .map_item(|_p,a| a
                   .add_style_back(TerminalStyle::bold(true))
                   .add_style_back(TerminalStyle::fg_color((200,0,0))))
        ),
        (Point2::new(1, 0),
         msg
         .map_item(|_p,a| a
                   .add_style_back(TerminalStyle::fg_color((180,180,180))))
        )
    ]);

    Message {
        addr: vec![],
        port: mb.get_port().flatten()
    }
}

pub fn make_warn(msg: OuterViewPort<dyn TerminalView>) -> Message {
    let mut mb = IndexBuffer::new();
    mb.insert_iter(vec![
        (Point2::new(0, 0),
         make_label("warning: ")
         .map_item(|_p,a| a
                   .add_style_back(TerminalStyle::bold(true))
                   .add_style_back(TerminalStyle::fg_color((200,200,0))))
        ),
        (Point2::new(1, 0),
         msg
         .map_item(|_p,a| a
                   .add_style_back(TerminalStyle::fg_color((180,180,180))))
        )
    ]);

    Message {
        addr: vec![],
        port: mb.get_port().flatten()
    }
}

pub fn make_todo(msg: OuterViewPort<dyn TerminalView>) -> Message {
    let mut mb = IndexBuffer::new();
    mb.insert_iter(vec![
        (Point2::new(0, 0),
         make_label("todo: ")
         .map_item(|_p,a| a
                   .add_style_back(TerminalStyle::bold(true))
                   .add_style_back(TerminalStyle::fg_color((180,180,250))))
        ),
        (Point2::new(1, 0),
         msg
         .map_item(|_p,a| a
                   .add_style_back(TerminalStyle::fg_color((180,180,180))))
        )
    ]);

    Message {
        addr: vec![],
        port: mb.get_port().flatten()
    }
}

pub fn make_info(msg: OuterViewPort<dyn TerminalView>) -> Message {
    let mut mb = IndexBuffer::new();
    mb.insert_iter(vec![
        (Point2::new(0, 0),
         make_label("info: ")
         .map_item(|_p,a| a
                   .add_style_back(TerminalStyle::bold(true))
                   .add_style_back(TerminalStyle::fg_color((180,180,250))))
        ),
        (Point2::new(1, 0),
         msg
         .map_item(|_p,a| a
                   .add_style_back(TerminalStyle::fg_color((180,180,180))))
        )
    ]);

    Message {
        addr: vec![],
        port: mb.get_port().flatten()
    }
}

