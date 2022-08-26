#[derive(Debug)]
pub enum VmError {
    Todo
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VmError::Todo => write!(f, "TODO"),
        }
    }
}

impl std::error::Error for VmError {}