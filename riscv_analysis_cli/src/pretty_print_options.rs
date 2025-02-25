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
    /// Output errors from all files.
    pub all_files: bool,
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
            all_files: false,
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

    /// Set the all_files option.
    pub fn all_files(mut self, value: bool) -> Self {
        self.all_files = value;
        self
    }
}
