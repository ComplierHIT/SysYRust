pub use std::collections::{HashSet, VecDeque};
pub use std::fs::File;
pub use std::hash::{Hash, Hasher};
pub use std::io::{Result, Write};

use crate::utility::{ObjPtr};
use crate::backend::operand::{IImm, FImm};
use crate::backend::instrs::{LIRInst};
use crate::backend::block::BB;


#[derive(Clone)]
pub struct IGlobalVar {
    name: String,
    init: bool,
    value: IImm,
}
#[derive(Clone)]
pub struct FGlobalVar {
    name: String,
    init: bool,
    value:FImm,
}

//TODO: to implement const array
#[derive(Clone)]
pub enum GlobalVar {
    IGlobalVar(IGlobalVar),
    FGlobalVar(FGlobalVar)
}

#[derive(Clone, PartialEq)]
pub struct StackSlot {
    pos: i32,
    size: i32,
}

pub struct Context {
    stack_offset: i32,
    epilogue: Option<Box<dyn FnMut()>>,
    prologue: Option<Box<dyn FnMut()>>,
}



#[derive(Clone)]
pub struct CurInstrInfo {
    block: Option<ObjPtr<BB>>,
    insts_it: Vec<ObjPtr<LIRInst>>,
    reg_id: i32,
}

impl Context {
    pub fn new() -> Self {
        Self {
            stack_offset: 0,
            epilogue: None,
            prologue: None,
        }
    }

    pub fn set_epilogue_event<F: FnMut() + 'static>(&mut self, callback: F) {
        self.epilogue = Some(Box::new(callback));
    }
    
    pub fn set_prologue_event<F: FnMut() + 'static>(&mut self, callback: F) {
        self.prologue = Some(Box::new(callback));
    }

    pub fn set_offset(&mut self, offset: i32) {
        self.stack_offset = offset;
    }

    pub fn get_offset(&self) -> i32 {
        self.stack_offset
    }

    pub fn call_epilogue_event(&mut self) {
        if let Some(ref mut callback) = self.epilogue {
            callback();
        }
    }

    pub fn call_prologue_event(&mut self) {
        if let Some(ref mut callback) = self.prologue {
            callback();
        }
    }
}

impl CurInstrInfo {
    pub fn new(reg_id: i32) -> Self {
        Self {
           reg_id,
           block: None,
           insts_it: Vec::new(),
        }
    }

    pub fn band_block(&mut self, block: ObjPtr<BB>) {
        self.block = Some(block);
    }

    pub fn get_block(&self) -> Option<ObjPtr<BB>> {
        self.block
    }

    pub fn add_inst(&mut self, inst: ObjPtr<LIRInst>) {
        self.insts_it.push(inst.clone());
    }

    pub fn add_insts(&mut self, insts: Vec<ObjPtr<LIRInst>>) {
        self.insts_it.append(&mut insts.clone());
    }
}

impl IGlobalVar {
    pub fn init(name: String, value: i32, init: bool) -> Self {
        Self { name, value: IImm::new(value), init }
    }
    pub fn new(name: String) -> Self {
        Self::init(name, 0, false)
    }
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_init(&self) -> IImm {
        self.value
    }
}

impl FGlobalVar {
    pub fn init(name: String, value: f32, init: bool) -> Self {
        Self { name, value: FImm::new(value), init }
    }
    pub fn new(name: String) -> Self {
        Self::init(name, 0.0, false)
    }
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_init(&self) -> FImm {
        self.value
    }
}

pub trait GenerateAsm {
    fn generate(&self, _: ObjPtr<Context>, f: &mut File) -> Result<()> {
        writeln!(f, "unreachable")?;
        Ok(())
    }
}

impl PartialEq for CurInstrInfo {
    fn eq(&self, other: &Self) -> bool {
        self.reg_id == other.reg_id
    }
}

impl Eq for CurInstrInfo {}

impl Hash for CurInstrInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.reg_id.hash(state);
    }
}

impl StackSlot {
    pub fn new(pos: i32, size: i32) -> Self {
        Self{ pos, size }
    }
    pub fn get_pos(&self) -> i32 {
        self.pos
    }
    pub fn get_size(&self) -> i32 {
        self.size
    }

    pub fn set_pos(&mut self, pos: i32) {
        self.pos = pos
    } 
    fn set_size(&mut self, size: i32) {
        self.size = size
    }
}   

impl PartialEq for BB {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

impl Eq for BB {}

impl Hash for BB {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.label.hash(state);
    }
}