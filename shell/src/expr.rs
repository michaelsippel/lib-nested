use {
    std::{
        sync::{Arc, RwLock}
    }
    nested::{
        core::TypeTerm,
    }
};

struct ExprEditor {
    editor: Arc<RwLock<dyn TerminalTreeEditor>>,
    type_tag: TypeTerm
}

impl TreeNav for ExprEditor {
    
}

impl TerminalEditor for ExprEditor {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        
    }
}

