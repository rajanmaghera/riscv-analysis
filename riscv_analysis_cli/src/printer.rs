use std::fs;

use bat::line_range::{LineRange, LineRanges};
use bat::{Input, PrettyPrinter};
use colored::Colorize;
use serde_json::{json, Value};

use riscv_analysis::parser::{Position, RVParser};
use riscv_analysis::passes::{DiagnosticItem, SeverityLevel};
use riscv_analysis::reader::FileReader;

pub trait ErrorDisplay {
    fn display_errors<T: FileReader + Clone>(&self, parser: &RVParser<T>);
}

/// Pretty printer for errors.
pub struct PrettyPrint {
    diagnostics: Vec<DiagnosticItem>,
}

impl PrettyPrint {
    pub fn new(errors: Vec<DiagnosticItem>) -> Self {
        Self {
            diagnostics: errors,
        }
    }
}

impl ErrorDisplay for PrettyPrint {
    fn display_errors<T: FileReader + Clone>(&self, parser: &RVParser<T>) {
        for err in &self.diagnostics {
            let filename = parser
                .reader
                .get_filename(err.file)
                .unwrap_or("unknown".to_owned());

            let level = match err.level {
                SeverityLevel::Error => "ERROR",
                SeverityLevel::Warning => "WARNING",
                SeverityLevel::Information => "INFO",
                SeverityLevel::Hint => "HINT",
            };

            if let Some(text) = parser.reader.get_text(err.file) {
                PrettyPrinter::new()
                    .input(
                        Input::from_reader(text.as_bytes())
                            .kind(format!("{} in file", level))
                            .name(filename.clone()),
                    )
                    .header(true)
                    .line_numbers(true)
                    .grid(true)
                    .paging_mode(bat::PagingMode::Never)
                    .line_ranges(LineRanges::from(vec![LineRange::new(
                        err.range.start.line + 1,
                        err.range.start.line + 1,
                    )]))
                    .print()
                    .unwrap();

                // print range arrows
                println!(
                    "       {}",
                    " ".repeat(err.range.start.column)
                        + &"^".repeat(err.range.end.column - err.range.start.column)
                );
            }

            print!(
                "       {}\n       {}\n",
                err.title.bold(),
                err.long_description
            );
        }
    }
}

/// Print lints as JSON
pub struct JSONPrint {
    diagnostics: Vec<DiagnosticItem>,
}

impl JSONPrint {
    /// Create a new JSON printer.
    pub fn new(errors: Vec<DiagnosticItem>) -> Self {
        Self {
            diagnostics: errors,
        }
    }

    /// Convert a single diagnostic item to JSON
    fn to_json<T: FileReader + Clone> (&self, parser: &RVParser<T>, item: &DiagnosticItem) -> Value {
        // Get the fields
        let path = parser
            .reader
            .get_filename(item.file)
            .unwrap_or("unknown".to_owned());
        let path = fs::canonicalize(path)
            .unwrap();
        let level = match item.level {
            SeverityLevel::Error => "Error",
            SeverityLevel::Warning => "Warning",
            SeverityLevel::Information => "Info",
            SeverityLevel::Hint => "Hint",
        };

        // Convert to JSON
        json!({
            "file": path,
            "title": item.title,
            "description": item.description,
            "level": level,
            "range": {
                "start" : self.position_to_json(item.range.start),
                "end" : self.position_to_json(item.range.end),
            }
        })
    }

    fn position_to_json(&self, pos: Position) -> Value {
        json!({
            "line": pos.line,
            "column": pos.column,
            "raw": pos.raw_index,
        })
    }
}

impl ErrorDisplay for JSONPrint {
    fn display_errors<T: FileReader + Clone>(&self, parser: &RVParser<T>) {
        // Convert the diagnostic items to JSON
        let sub: Vec<_> = self
            .diagnostics
            .iter()
            .map(|d| self.to_json(parser, d))
            .collect();

        // Print the results
        let out = json!({ "diagnostics": sub });
        let text = serde_json::to_string_pretty(&out).unwrap();
        println!("{}", text);
    }
}
