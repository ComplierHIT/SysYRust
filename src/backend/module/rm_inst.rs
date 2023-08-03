use super::*;

impl AsmModule {
    pub fn rm_inst_suf_p2v(
        &mut self,
        pool: &mut BackendPool,
        callers_used: &HashMap<String, HashSet<Reg>>,
        callees_used: &HashMap<String, HashSet<Reg>>,
        callees_saved: &HashMap<String, HashSet<Reg>>,
    ) {
        for (_, func) in self.name_func.iter() {
            func.as_mut().remove_unuse_inst_suf_v2p(
                pool,
                callers_used,
                callees_used,
                callees_saved,
            );
        }
    }
}
