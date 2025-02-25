/// Options for pretty printing.
///
/// This struct is used to configure the output of the pretty printer.
/// Its methods do not modify the options, but instead is used
/// to hold the configuration for the pretty printer.
pub struct PrettyPrintOptions {
    /// Compact one-line output.
    pub compact: bool,
    /// Enable color output.
    pub color: bool,
}

impl Default for PrettyPrintOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl PrettyPrintOptions {
    /// Create a new pretty print options.
    pub fn new() -> Self {
        Self {
            compact: false,
            color: true,
        }
    }

    /// Set the compact option.
    pub fn compact(mut self, value: bool) -> Self {
        self.compact = value;
        self
    }

    /// Set the color option.
    pub fn color(mut self, value: bool) -> Self {
        self.color = value;
        self
    }
}
