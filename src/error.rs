#[derive(Debug)]
pub enum VmError {
    FunctionDoesNotExist(usize),
    SymbolDoesNotExist { func : usize, sym : usize },
    RedefinitionOfLabel { func : usize, label : usize },
    LabelDoesNotExist { func : usize, label : usize },
    ReturnNotSet { func : usize, sym : usize },
    AttemptToCallNonFunction { current_func : usize },
    AttemptToPopEmptyParams { current_func : usize, sym : usize },
    AttemptToFreeUnallocatedAddress { current_func : usize, sym : usize, address : (u64, u64) },
    AttemptToFreeValue { current_func: usize, sym : usize },
    AttemptToFreeFunc { current_func: usize, sym : usize },
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VmError::SymbolDoesNotExist { func, sym } => write!(f, "symbol {} does not exist for function {}", sym, func),
            VmError::FunctionDoesNotExist(func) => write!(f, "symbol does not exist:  {}", func),
            VmError::RedefinitionOfLabel { func, label } => write!(f, "redefinition of label {} in function {}", label, func),
            VmError::LabelDoesNotExist { func, label } => write!(f, "label {} does not exist in function {}", label, func),
            VmError::ReturnNotSet { func, sym } => 
                write!(f, "return not set in function {} for set into symbol {}", func, sym),
            VmError::AttemptToCallNonFunction { current_func } => 
                write!(f, "attempt to call non-function in function {}", current_func),
            VmError::AttemptToPopEmptyParams { current_func, sym } =>
                write!(f, "attempt to pop empty params in function {} into symbol {}", current_func, sym),
            VmError::AttemptToFreeUnallocatedAddress { current_func, sym, address: (session, element) } => 
                write!(f, "attempt to free unallocated address: function {}: symbol {}: address {}#{}"
                        , current_func, sym, session, element),
            VmError::AttemptToFreeValue { current_func, sym } =>
                write!(f, "attempt to free data in function {} from symbol {}", current_func, sym),
            VmError::AttemptToFreeFunc { current_func, sym } => 
                write!(f, "attempt to free func in function {} from symbol {}", current_func, sym),
        }
    }
}

impl std::error::Error for VmError {}