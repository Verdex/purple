#[derive(Debug)]
pub enum VmError {
    FunctionDoesNotExist(usize),
    SymbolDoesNotExist(usize),
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VmError::SymbolDoesNotExist(sym) => write!(f, "symbol does not exist:  {}", sym),
            VmError::FunctionDoesNotExist(func) => write!(f, "symbol does not exist:  {}", func),
        }
    }
}

impl std::error::Error for VmError {}