use std::{collections::HashMap, sync::LazyLock};

use rmachine_core::machine::HandlerMap;

use crate::{IRQ, IRQ_VECTOR, NMI, NMI_VECTOR, RESET, RESET_VECTOR};

pub static HANDLERS: LazyLock<HandlerMap> = LazyLock::new(|| {
    let mut handlers: HandlerMap = HashMap::new();
    handlers.insert(RESET, |m, mem| {
        m.pc = mem.get16(RESET_VECTOR);
        m.step(mem);
    });
    handlers.insert(IRQ, |m, mem| {
        m.pc = mem.get16(IRQ_VECTOR);
        m.step(mem);
    });
    handlers.insert(NMI, |m, mem| {
        m.pc = mem.get16(NMI_VECTOR);
        m.step(mem);
    });
    handlers
});
