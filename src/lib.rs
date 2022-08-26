
mod error;
mod data;

use crate::error::VmError;
use crate::data::*;

struct Frame<'a, T : Clone + ToData<'a, T>> {
    instr_ptr : usize,
    locals : Locals<'a, T>,
    current_function : usize,
}


pub fn run<'a, T : Clone + ToData<'a, T>>( func_defs : &'a Vec<FuncDef<'a, T>>
                                         , heap : &'a mut Vec<Data<'a, T>> 
                                         ) 
                                         -> Result<Data<'a, T>, Box<dyn std::error::Error>> {

    if func_defs.len() == 0 {
        return Err(Box::new(VmError::FunctionDoesNotExist(0)));
    }

    let mut stack : Vec<Frame<'a, T>> = vec![];
    let mut current_function = 0;
    let mut instrs = &func_defs[current_function].body;
    let mut instr_ptr = 0;
    let mut locals : Locals<'a, T> = Locals::new();

    loop {

        if instrs.len() <= instr_ptr {
            if stack.len() == 0 {
                break;
            }

            Frame { instr_ptr, locals, current_function } = stack.pop().unwrap();
            // NOTE:  We don't have to check if current_function exists because if we're poping
            // then we must have called it previously.
            instrs = &func_defs[current_function].body;
        }

        /*match instrs[instr_ptr] {
            Instr::Label(Label),
            Instr::Jump(Label),
            Instr::BranchOnTrue(Label, Box<dyn FnMut(&Locals<'a, T>, &'a Vec<Data<'a, T>>) -> Result<bool, Box<dyn std::error::Error>>>),
            Instr::Return(Symbol),
            Instr::LoadValue(Symbol, T),
            Instr::LoadFromReturn(Symbol),
            Instr::PushParam(Symbol),
            Instr::LoadFromExec(Symbol, Box<dyn FnMut(&Locals<'a, T>) -> Result<Data<'a, T>, Box<dyn std::error::Error>>>),
            Instr::LoadFunc(Symbol, Func),
            Instr::Call(Symbol),
            Instr::SysCall(Box<dyn FnMut(&Locals<'a, T>, &'a Vec<Data<'a, T>>) -> Result<(), Box<dyn std::error::Error>>>),
            Instr::LoadFromSysCall(Symbol, Box<dyn FnMut(&Locals<'a, T>, &'a Vec<Data<'a, T>>) -> Result<Data<'a, T>, Box<dyn std::error::Error>>>),
        }*/

    }

    Err(Box::new(VmError::SymbolDoesNotExist(0)))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
