/// Flags used in the status register.
///
/// 7 6 5 4 3 2 1 0
/// N V - B D I Z C
pub const CARRY: u8 = 0b0000_0001;
pub const ZERO: u8 = 0b0000_0010;
pub const INTERRUPT: u8 = 0b0000_0100;
pub const DECIMAL: u8 = 0b0000_1000;
