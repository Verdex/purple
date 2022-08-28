
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
    let mut params : Vec<Data<'a, T>> = vec![];
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
            Instr::Call(sym) => {
                match locals.get(sym)? {
                    Data::Func(f) => {

                        if func_defs.len() <= f.0 {
                            return Err(Box::new(VmError::FunctionDoesNotExist(f.0)));
                        }

                        let old_function = current_function;
                        let mut old_instrs = &func_defs[f.0].body;
                        let old_instr_ptr = instr_ptr;
                        let mut old_locals : Locals<'a, T> = Locals::new(f.0);
                        let mut old_label_map : HashMap<Label, usize> = HashMap::new();

                        current_function = f.0;
                        instr_ptr = 0;
                        std::mem::swap(&mut old_instrs, &mut instrs);
                        std::mem::swap(&mut old_locals, &mut locals);
                        std::mem::swap(&mut old_label_map, &mut label_map);

                        stack.push(Frame { instr_ptr: old_instr_ptr
                                         , locals: old_locals
                                         , current_function: old_function
                                         , label_map: old_label_map
                                         });
                    },
                    _ => return Err(Box::new(VmError::AttemptToCallNonFunction { current_func: current_function })),
                }

            },
            Instr::PushParam(sym) => {
                params.push(locals.get(sym)?);
                instr_ptr += 1;
            },
            Instr::PopParam(sym) => {
                match params.pop() {
                    Some(param) => locals.set(sym, param)?,
                    None => return Err(Box::new(VmError::AttemptToPopEmptyParams { current_func: current_function, sym: sym.0 })),
                }
                instr_ptr += 1;
            },
            Instr::LoadFromExec(sym, f) => {
                locals.set(sym, f(&locals)?)?;
                instr_ptr += 1;
            },
            Instr::LoadFunc(sym, f) => {
                locals.set(sym, Data::Func(*f))?;
                instr_ptr += 1;
            },
            /*Instr::Alloc { dest, contents } => {

            },
            Free(Symbol),
            Store { address: Symbol, contents : Symbol },
            Get { address: Symbol, dest: Symbol },
            */
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
