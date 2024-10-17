use std::collections::HashMap;
use std::fs;

use colored::Colorize;
use serde_json::{json, Value};

use riscv_analysis::parser::{Position, RVParser};
use riscv_analysis::passes::{DiagnosticItem, SeverityLevel};
use riscv_analysis::reader::FileReader;
use uuid::Uuid;

pub trait ErrorDisplay {
    fn display_errors<T: FileReader + Clone>(&mut self, parser: &RVParser<T>);
}

/// Pretty printer for errors.
pub struct PrettyPrint {
    diagnostics: Vec<DiagnosticItem>,
    files: HashMap<Uuid, Vec<String>>,   // Cache loaded files
}

impl PrettyPrint {
    pub fn new(errors: Vec<DiagnosticItem>) -> Self {
        Self {
            diagnostics: errors,
            files: HashMap::new(),
        }
    }

    /// Return the contents of a file, caching the results.
    fn get_file<T: FileReader + Clone>(&mut self, parser: &RVParser<T>, file: &Uuid) -> Option<&Vec<String>> {
        // Load the file if we haven't already
        if !self.files.contains_key(file) {
            let path = parser.reader.get_filename(*file)?;
            let contents = fs::read_to_string(path).ok()?;
            let lines: Vec<String> = contents
                .split('\n')
                .map(|s| s.to_string())
                .collect();
            self.files.insert(*file, lines);
        }

        // Return the file contents
        let contents = self.files.get(file)?;
        Some(contents)
    }

    /// Get the region associated with LINE.
    fn get_line(contents: &[String], line: usize) -> Option<&String> {
        contents.get(line)
    }

    /// Return the name of a severity level.
    fn level(&self, level: &SeverityLevel) -> String {
        match level {
            SeverityLevel::Error => "Error".red(),
            SeverityLevel::Warning => "Warning".yellow(),
            SeverityLevel::Information => "Info".blue(),
            SeverityLevel::Hint => "Hint".green(),
        }.bold().to_string()
    }

    /// Compute A - B, and return OR if this overflows.
    fn sub_or(a: usize, b: usize, or: usize) -> usize {
        match a.checked_sub(b) {
            Some(diff) => diff,
            None => or,
        }
    }

    /// Format the source region portion of the message.
    fn format_region(text: &str, line: usize, start: usize, end: usize) -> String {
        // Compute the space needed for the line number
        let line = line + 1;
        let n_spc = line.to_string().len() + 1;
        let spc = " ".repeat(n_spc);

        // Left align the text
        let mut first_non_ws = 0;
        for (i, c) in text.chars().enumerate() {
            if !c.is_whitespace() {
                first_non_ws = i;
                break;
            }
        }

        // Arrows pointing the the relevant position
        let end = end + 1;
        let aligned = text.trim();
        let arrows = "^".repeat(Self::sub_or(end, start, 0));
        let arrow_spc = " ".repeat(Self::sub_or(start, first_non_ws, 0));

        format!("{spc} |\n {line} | {aligned}\n{spc} | {arrow_spc}{arrows}\n")
    }

    /// Fromat a diagnostic item.
    fn format_item<T: FileReader + Clone>(&mut self, parser: &RVParser<T>, item: &DiagnosticItem) -> String {
        let level = self.level(&item.level);
        let title = &item.title;
        let path = parser.reader
                         .get_filename(item.file)
                         .unwrap_or("<unknown file>".to_string());

        // Print the name of the error & file
        let mut acc = format!(
            "{level}: {title}\n in file: {path}\n"
        );

        // Print the relevant region
        if let Some(text) = self.get_file(parser, &item.file) {
            let line = item.range.start.line;
            if let Some(region) = Self::get_line(text, line) {
                let start = item.range.start.column;
                let end = item.range.end.column;
                acc.push_str(&Self::format_region(region, line, start, end));
            }
        }

        acc.push('\n');
        acc
    }
}

impl ErrorDisplay for PrettyPrint {
    fn display_errors<T: FileReader + Clone>(&mut self, parser: &RVParser<T>) {
        for err in self.diagnostics.clone() {
            let out = self.format_item(parser, &err);
            print!("{}", out);
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
    fn display_errors<T: FileReader + Clone>(&mut self, parser: &RVParser<T>) {
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
