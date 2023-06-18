use std::collections::HashSet;

use crate::{
    ir::{basicblock::BasicBlock, instruction::InstKind, module::Module},
    utility::ObjPtr,
};

///! 对于block的优化
///! 1. 删除无法到达的block：除头block外没有前继的就是无法到达的
///! 2. 合并只有一个后继和这个后继只有一个前继的block
///! 3. 删除无法到达的分支

pub fn simplify_cfg_run(module: &mut Module) {
    for (_, func) in module.get_all_func().iter() {
        if func.is_empty_bb() || !func.get_head().has_next_bb() {
            continue;
        }

        remove_unreachable_bb(func.get_head());
    }
}

fn merge_one_line_bb(head: ObjPtr<BasicBlock>) {}

fn remove_unreachable_bb(head: ObjPtr<BasicBlock>) {
    let mut deleted = HashSet::new();

    let bb_list = get_bb_list(head);

    loop {
        let mut changed = false;
        for bb in bb_list.iter() {
            // 不考虑头和尾
            if bb.clone() == head || bb_is_end(bb.clone()) {
                continue;
            }

            // 如果没有前继或者前继都在deleted集里，那么当前bb是无法到达的
            if bb.get_up_bb().is_empty() || bb.get_up_bb().iter().all(|bb| deleted.contains(bb)) {
                deleted.insert(bb);
                changed = true;
            }

            // jump指令不检查
            if bb_has_jump(bb.clone()) {
                continue;
            }

            // 检查是否分支不可达，并删除掉不可达的路径
            changed |= check_bb(bb.clone());
        }

        if !changed {
            break;
        }
    }

    // 删除掉这些不可达的bb
    for &bb in deleted.iter() {
        let should_be_deleted: Vec<&ObjPtr<BasicBlock>> = bb
            .get_next_bb()
            .iter()
            .filter(|x| !deleted.contains(x))
            .collect();

        for &next_bb in should_be_deleted.iter() {
            bb.as_mut().remove_next_bb(next_bb.clone());
        }

        remove_bb_self(bb.clone());
    }
}

fn remove_bb_self(bb: ObjPtr<BasicBlock>) {
    if bb.is_empty() {
        return;
    }
    let mut inst = bb.get_head_inst();
    loop {
        let next = inst.get_next();
        inst.remove_self();
        inst = next;
        if inst.is_tail() {
            inst.remove_self();
            break;
        }
    }
}

/// 检查分支是否无法到达
/// 如果无法到达，那么删除到达这个分支的路径
fn check_bb(bb: ObjPtr<BasicBlock>) -> bool {
    let mut changed = false;
    let cond = bb.get_tail_inst().get_br_cond();

    match cond.get_kind() {
        InstKind::ConstInt(value) => {
            if value == 0 {
                bb.as_mut().remove_next_bb(bb.get_next_bb()[1].clone());
            } else {
                bb.as_mut().remove_next_bb(bb.get_next_bb()[0].clone());
            }
            changed = true;
        }
        InstKind::ConstFloat(value) => {
            if value == 0.0 {
                bb.as_mut().remove_next_bb(bb.get_next_bb()[1].clone());
            } else {
                bb.as_mut().remove_next_bb(bb.get_next_bb()[0].clone());
            }
            changed = true;
        }

        _ => {}
    }

    changed
}

fn get_bb_list(head: ObjPtr<BasicBlock>) -> Vec<ObjPtr<BasicBlock>> {
    let mut queue = Vec::new();
    let mut visited = HashSet::new();
    queue.insert(0, head);
    while let Some(bb) = queue.pop() {
        if !visited.contains(&bb) {
            visited.insert(bb.clone());
            queue.extend(bb.get_next_bb().iter().cloned());
        }
    }
    visited.iter().cloned().collect::<Vec<_>>()
}

fn bb_is_end(bb: ObjPtr<BasicBlock>) -> bool {
    bb.get_next_bb().is_empty()
}

fn bb_has_jump(bb: ObjPtr<BasicBlock>) -> bool {
    bb.get_tail_inst().is_jmp()
}