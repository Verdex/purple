
use std::collections::HashMap;

mod error;
mod data;

use crate::error::VmError;
use crate::data::*;

struct Frame<'a, T : Clone + ToData<'a, T>> {
    instr_ptr : usize,
    locals : Locals<'a, T>,
    current_function : usize,
    label_map : HashMap<Label, usize>,
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
    let mut locals : Locals<'a, T> = Locals::new(current_function);
    let mut label_map : HashMap<Label, usize> = HashMap::new();
    let mut ret = None;

    loop {

        if instrs.len() <= instr_ptr {
            if stack.len() == 0 {
                break;
            }

            Frame { instr_ptr, locals, current_function, label_map } = stack.pop().unwrap();
            // NOTE:  We don't have to check if current_function exists because if we're poping
            // then we must have called it previously.
            instrs = &func_defs[current_function].body;
        }

        match &instrs[instr_ptr] {
            Instr::Label(label) => 
                match label_map.insert( *label, instr_ptr + 1 ) {
                    Some(_) => return Err(Box::new(VmError::RedefinitionOfLabel { label : label.0, func : current_function})),
                    None => { instr_ptr += 1; },
                },
            Instr::Jump(label) =>
                match label_map.get(label) {
                    Some(ptr) => instr_ptr = *ptr,
                    None => return Err(Box::new(VmError::LabelDoesNotExist {label : label.0, func : current_function})),
                },
            Instr::BranchOnTrue(label, f) => {
                if f(&locals)? {
                    match label_map.get(label) {
                        Some(ptr) => instr_ptr = *ptr,
                        None => return Err(Box::new(VmError::LabelDoesNotExist {label : label.0, func : current_function})),
                    }
                }
                else {
                    instr_ptr += 1;
                }
            },
            Instr::Return(sym) => {
                ret = Some(locals.get(sym)?);

                if stack.len() == 0 {
                    break;
                }

                Frame { instr_ptr, locals, current_function, label_map } = stack.pop().unwrap();
                // NOTE:  We don't have to check if current_function exists because if we're poping
                // then we must have called it previously.
                instrs = &func_defs[current_function].body;
            },
            Instr::LoadValue(sym, data) => {
                locals.set(sym, Data::Value(data.clone()))?;
                instr_ptr += 1;
            },
            Instr::LoadFromReturn(sym) => {
                match ret {
                    Some(ref ret) => {
                        locals.set(sym, ret.clone())?;
                        instr_ptr += 1;
                    },
                    None => return Err(Box::new(VmError::ReturnNotSet { func: current_function, sym: sym.0 })),
                }
            },
            /*Instr::PushParam(Symbol),
            Instr::LoadFromExec(Symbol, Box<dyn FnMut(&Locals<'a, T>) -> Result<Data<'a, T>, Box<dyn std::error::Error>>>),
            Instr::LoadFunc(Symbol, Func),
            Instr::Call(Symbol),*/
            Instr::SysCall(f) => {
                f(&locals, heap)?;
                instr_ptr += 1;
            },
            Instr::LoadFromSysCall(sym, f) => {

            },
            _ => panic!("TODO remove"),
        }

    }

    Err(Box::new(VmError::SymbolDoesNotExist { func: 0, sym: 0 }))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
