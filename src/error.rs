#[derive(Debug)]
pub enum VmError {
    FunctionDoesNotExist(usize),
    SymbolDoesNotExist { func : usize, sym : usize },
    RedefinitionOfLabel { func : usize, label : usize },
    LabelDoesNotExist { func : usize, label : usize }
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VmError::SymbolDoesNotExist { func, sym } => write!(f, "symbol {} does not exist for function {}", sym, func),
            VmError::FunctionDoesNotExist(func) => write!(f, "symbol does not exist:  {}", func),
            VmError::RedefinitionOfLabel { func, label } => write!(f, "redefinition of label {} in function {}", label, func),
            VmError::LabelDoesNotExist { func, label } => write!(f, "label {} does not exist in function {}", label, func),
        }
    }
}

impl std::error::Error for VmError {}