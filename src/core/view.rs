
                    /*\
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                   View
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                    \*/
pub trait View : Send + Sync {
    /// Notification message for the observers
    type Msg;
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

use std::sync::{Arc, RwLock};

impl<V: View> View for RwLock<V> {
    type Msg = V::Msg;
}

impl<V: View> View for Arc<V> {
    type Msg = V::Msg;
}


