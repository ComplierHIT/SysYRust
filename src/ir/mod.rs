//! src/ir/mod.rs

use crate::utility::ObjPtr;
pub mod basicblock;
mod dump_ir;
pub mod function;
pub mod instruction;
pub mod ir_type;
pub mod module;
mod optimizer;
mod tools;
pub mod user;
pub mod value;

pub use dump_ir::dump_now;
pub use optimizer::optimizer_run;
pub use optimizer::{call_map_gen, CallMap};

/// 侵入式链表
#[derive(Debug, Clone)]
pub struct IList<T: 'static> {
    prev: Option<ObjPtr<T>>,
    next: Option<ObjPtr<T>>,
}

impl<T: 'static> IList<T> {
    pub fn set_prev(&mut self, value: ObjPtr<T>) {
        self.prev = Some(value);
    }

    pub fn set_next(&mut self, value: ObjPtr<T>) {
        self.next = Some(value);
    }

    pub fn get_prev(&self) -> ObjPtr<T> {
        if let Some(p) = self.prev {
            p
        } else {
            panic!("prev is none")
        }
    }

    pub fn get_next(&self) -> ObjPtr<T> {
        if let Some(p) = self.next {
            p
        } else {
            panic!("next is none")
        }
    }
}
