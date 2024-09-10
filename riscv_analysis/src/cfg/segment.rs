/// Enum representing the different segments of a RISC-V binary.
///
/// The segments are:
/// - `.text`: The text segment, which contains the instructions
/// - `.data`: The data segment, which contains the data
///
/// All instructions must be in the `.text` segment, and all data
/// must be in the `.data` segment. Jumping to instructions in
/// the `.data` segment is highly unlikely.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Segment {
    /// The `.text` segment containing the instructions
    Text,
    /// The `.data` segment containing binary data
    Data,
}
