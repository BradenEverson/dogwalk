//! Messages from the server to the raw hardware

/// Any message the hardware may receive
pub enum PuppyMsg {
    /// Move servo with `idx`(first argument) to `angle` degrees(second argument)
    MoveServe(u8, u16),
}
