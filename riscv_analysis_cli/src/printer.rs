use bat::line_range::{LineRange, LineRanges};
use bat::{Input, PrettyPrinter};
use colored::Colorize;
use riscv_analysis::parser::RVParser;
use riscv_analysis::passes::{DiagnosticItem, SeverityLevel};
use riscv_analysis::reader::FileReader;

pub trait ErrorDisplay {
    fn display_errors<T: FileReader + Clone>(&self, parser: &RVParser<T>);
}

impl ErrorDisplay for Vec<DiagnosticItem> {
    fn display_errors<T: FileReader + Clone>(&self, parser: &RVParser<T>) {
        for err in self {
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
