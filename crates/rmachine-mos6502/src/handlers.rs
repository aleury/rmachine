use std::{collections::HashMap, sync::LazyLock};

use rmachine_core::machine::HandlerMap;

use crate::{IRQ, IRQ_VECTOR, NMI, NMI_VECTOR, RESET, RESET_VECTOR};

pub static HANDLERS: LazyLock<HandlerMap> = LazyLock::new(|| {
    let mut handlers: HandlerMap = HashMap::new();
    handlers.insert(RESET, |m| {
        m.pc = m.get16(RESET_VECTOR);
        m.step();
    });
    handlers.insert(IRQ, |m| {
        m.pc = m.get16(IRQ_VECTOR);
        m.step();
    });
    handlers.insert(NMI, |m| {
        m.pc = m.get16(NMI_VECTOR);
        m.step();
    });
    handlers
});
