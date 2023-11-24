

impl PtySegment for SumEditor {
    fn pty_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.port.outer()
    }
}

