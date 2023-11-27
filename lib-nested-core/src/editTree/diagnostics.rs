use {
    r3vi::{
        view::{OuterViewPort, sequence::*},
        buffer::{vec::*, index_hashmap::*}
    },
    crate::{
        reprTree::ReprTree
    },
    std::sync::{Arc, RwLock},
    cgmath::Point2
};

#[derive(Clone)]
pub struct Message {
    pub addr: Vec<usize>,
    pub disp: Arc<RwLock<ReprTree>>
}

pub trait Diagnostics {
    fn get_msg_port(&self) -> OuterViewPort<dyn SequenceView<Item = Message>> {
        VecBuffer::new().get_port().to_sequence()
    }
}

