pub trait ErrorPresentation {
    fn summary(&self) -> &str;

    fn detail(&self) -> &str;

    fn has_diagnostic_detail(&self) -> bool {
        self.detail() != self.summary()
    }
}

pub trait ResourceStatusPresentation {
    fn is_refreshing(&self) -> bool;

    fn has_value(&self) -> bool;

    fn error(&self) -> Option<&dyn ErrorPresentation>;
}

pub trait OperationStatusPresentation {
    fn is_running(&self) -> bool;

    fn error(&self) -> Option<&dyn ErrorPresentation>;
}
