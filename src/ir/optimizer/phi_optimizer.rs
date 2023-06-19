use std::collections::HashSet;

use crate::{
    ir::{basicblock::BasicBlock, instruction::InstKind},
    utility::ObjPtr,
};

///! 对于phi的优化主要针对以下几个方面：
///! 1. phi的参数只有一个时，直接替换为该参数;
///! 2. phi的多个参数相同时，也可以将其消去。

pub fn phi_run(end: ObjPtr<BasicBlock>) {
    loop {
        let mut changed = false;

        let mut visited = HashSet::new();
        let mut queue = Vec::new();
        queue.insert(0, end);

        // 往回广度优先遍历
        while let Some(bb) = queue.pop() {
            if !visited.contains(&bb) {
                visited.insert(bb);
                changed = changed || phi_optimize(bb);

                for prev in bb.get_up_bb().iter() {
                    queue.insert(0, *prev);
                }
            }
        }

        if !changed {
            break;
        }
    }
}

fn phi_optimize(bb: ObjPtr<BasicBlock>) -> bool {
    let mut changed = false;
    if bb.is_empty() {
        return changed;
    }
    let mut inst = bb.get_head_inst();
    while let InstKind::Phi = inst.get_kind() {
        if inst
            .get_operands()
            .iter()
            .all(|&x| x == inst.get_operands()[0])
        {
            changed = true;
            // 将phi指令替换为第一个参数
            while inst.get_use_list().len() != 0 {
                let mut user = inst.get_use_list()[0];
                let index = user.get_operands().iter().position(|x| x == &inst).unwrap();
                user.set_operand(inst.get_operands()[0], index);
            }
            inst = inst.get_next();
            inst.get_prev().remove_self();
        }
        inst = inst.get_next();
    }

    changed
}
