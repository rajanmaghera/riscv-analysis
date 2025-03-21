use std::collections::HashMap;
use std::fs;

use colored::Colorize;

use riscv_analysis::parser::RVParser;
use riscv_analysis::passes::{DiagnosticItem, SeverityLevel};
use riscv_analysis::reader::FileReader;
use uuid::Uuid;

use riscv_analysis_cli::wrapper::{DiagnosticTestCase, TestCase};

use crate::pretty_print_options::PrettyPrintOptions;

pub trait ErrorDisplay {
    fn display_errors<T: FileReader>(&mut self, parser: &RVParser<T>);
}

/// Pretty printer for errors.
pub struct PrettyPrint {
    diagnostics: Vec<DiagnosticItem>,
    files: HashMap<Uuid, Vec<String>>, // Cache loaded files
    options: PrettyPrintOptions,
}

impl PrettyPrint {
    pub fn new(errors: Vec<DiagnosticItem>, options: PrettyPrintOptions) -> Self {
        Self {
            diagnostics: errors,
            files: HashMap::new(),
            options,
        }
    }

    /// Return the contents of a file, caching the results.
    fn get_file<T: FileReader>(
        &mut self,
        parser: &RVParser<T>,
        file: &Uuid,
    ) -> Option<&Vec<String>> {
        // Load the file if we haven't already
        if !self.files.contains_key(file) {
            let path = parser.reader.get_filename(*file)?;
            let contents = fs::read_to_string(path).ok()?;
            let lines: Vec<String> = contents.split('\n').map(|s| s.to_string()).collect();
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
    /// Return the name of a severity level
    fn level_string(&self, level: &SeverityLevel) -> String {
        if self.options.color {
            match level {
                SeverityLevel::Error => "Error".red(),
                SeverityLevel::Warning => "Warning".yellow(),
                SeverityLevel::Information => "Info".blue(),
                SeverityLevel::Hint => "Hint".green(),
            }
            .bold()
            .to_string()
        } else {
            match level {
                SeverityLevel::Error => "Error",
                SeverityLevel::Warning => "Warning",
                SeverityLevel::Information => "Info",
                SeverityLevel::Hint => "Hint",
            }
            .to_string()
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

        // HACK: Use the text line so we have the same tab spacing
        let mut base: String = text
            .get(first_non_ws..)
            .unwrap_or_default()
            .chars()
            .map(|c| if c.is_whitespace() { c } else { ' ' })
            .collect();

        // Arrows pointing the the relevant position
        let end = end + 1;
        let arrows = "^".repeat(end.saturating_sub(start));
        let offset = start.saturating_sub(first_non_ws);
        base.replace_range(offset.., &arrows);

        let aligned = text.trim();
        format!("{spc} |\n {line} | {aligned}\n{spc} | {base}\n")
    }

    /// Format a diagnostic item in a compact (one-line) form.
    fn format_item_compact<T: FileReader>(
        &mut self,
        parser: &RVParser<T>,
        item: &DiagnosticItem,
    ) -> String {
        let level = self.level_string(&item.level);
        let title = &item.title;
        let path = parser
            .reader
            .get_filename(item.file)
            .unwrap_or("<unknown file>".to_string());
        let start = item.range.start().one_idx_line();
        let start_col = item.range.start().one_idx_column();
        let end_col = item.range.end().one_idx_column();

        format!("{level}: {title} in {path} at {start} {start_col}:{end_col}\n",)
    }

    /// Format a diagnostic item.
    fn format_item<T: FileReader>(
        &mut self,
        parser: &RVParser<T>,
        item: &DiagnosticItem,
    ) -> String {
        let level = self.level_string(&item.level);
        let title = &item.title;
        let path = parser
            .reader
            .get_filename(item.file)
            .unwrap_or("<unknown file>".to_string());

        // Print the name of the error & file
        let mut acc = format!("{level}: {title}\n in file: {path}\n");

        // Print the relevant source region
        if let Some(text) = self.get_file(parser, &item.file) {
            let line = item.range.start().zero_idx_line();
            if let Some(region) = Self::get_line(text, line) {
                let start = item.range.start().zero_idx_column();
                let end = item.range.end().zero_idx_column();
                acc.push_str(&Self::format_region(region, line, start, end));
            }
        }

        acc.push('\n');
        acc
    }
}

impl ErrorDisplay for PrettyPrint {
    fn display_errors<T: FileReader>(&mut self, parser: &RVParser<T>) {
        let mut errors_in_other_files = 0;
        for err in &self.diagnostics.clone() {
            if let Some(base_file) = parser.reader.get_base_file() {
                if err.file != base_file && !self.options.all_files {
                    errors_in_other_files += 1;
                    continue;
                }
            }
            if self.options.compact {
                let out = self.format_item_compact(parser, err);
                print!("{}", out);
            } else {
                let out = self.format_item(parser, err);
                print!("{}", out);
            }
        }
        if errors_in_other_files > 0 {
            let end_str = if errors_in_other_files > 1 { "s" } else { "" };
            println!("{} diagnostic{} found in other files. To see all errors, run with the `--all-files` option.", errors_in_other_files, end_str);
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
    fn wrap_item<T: FileReader>(
        &self,
        parser: &RVParser<T>,
        item: &DiagnosticItem,
    ) -> DiagnosticTestCase {
        // Get the fields
        let path = parser
            .reader
            .get_filename(item.file)
            .map(|f| fs::canonicalize(f).unwrap_or_default())
            .map(|p| p.to_str().unwrap_or_default().to_string());
        let level = match item.level {
            SeverityLevel::Error => "Error",
            SeverityLevel::Warning => "Warning",
            SeverityLevel::Information => "Info",
            SeverityLevel::Hint => "Hint",
        };

        DiagnosticTestCase {
            file: path,
            title: item.title.clone(),
            description: item.description.clone(),
            level: level.to_string(),
            range: item.range.clone().into(),
        }
    }
}

impl ErrorDisplay for JSONPrint {
    fn display_errors<T: FileReader>(&mut self, parser: &RVParser<T>) {
        // Convert the diagnostic items to JSON
        let sub: Vec<_> = self
            .diagnostics
            .iter()
            .map(|d| self.wrap_item(parser, d))
            .collect();

        // Print the results
        let out = TestCase { diagnostics: sub };
        let text = serde_json::to_string_pretty(&out).unwrap();
        println!("{}", text);
    }
}
