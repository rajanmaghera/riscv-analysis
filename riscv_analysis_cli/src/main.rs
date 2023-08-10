use std::vec;
use std::{collections::HashMap, iter::Peekable, str::FromStr};

use bat::line_range::{LineRange, LineRanges};
use bat::{Input, PrettyPrinter};
use colored::Colorize;
use riscv_analysis::cfg::Cfg;
use riscv_analysis::parser::{Info, LabelString, Lexer, RVParser, With};
use riscv_analysis::passes::DiagnosticItem;
use std::path::PathBuf;
use uuid::Uuid;

use riscv_analysis::passes::{DiagnosticLocation, Manager};

use clap::{Args, Parser, Subcommand};
use riscv_analysis::reader::{FileReader, FileReaderError};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lint a file
    #[clap(name = "lint")]
    Lint(Lint),
    /// Fix known errors in a file
    ///
    /// This will attempt to fix known errors in a file.
    /// Known issues include incorrect stack saving, multiple returns, and mismatched register names.
    /// (not implemented)
    #[clap(name = "fix")]
    Fix(Fix),
    /// Debug options for testing
    #[clap(name = "debug_parse")]
    DebugParse(DebugParse),
}

#[derive(Args)]
struct Lint {
    /// Input file
    input: PathBuf,
    /// Debug mode
    #[clap(short, long)]
    debug: bool,
    /// Output debug as yaml
    #[clap(long)]
    yaml: bool,
    /// Remove output
    #[clap(long)]
    no_output: bool,
}

#[derive(Args)]
struct Fix {
    /// Input file
    ///
    /// This will attempt to fix known errors in a file.
    /// The file will be overwritten with the fixed version.
    input: PathBuf,
    /// Function name
    ///
    /// Name of a function to fix.
    func_name: String,
}

#[derive(Args)]
struct DebugParse {
    /// Input file
    input: PathBuf,
}

#[derive(Clone)]
struct IOFileReader {
    // path, uuid
    files: HashMap<uuid::Uuid, (String, String)>,
}

impl IOFileReader {
    fn new() -> Self {
        IOFileReader {
            files: HashMap::new(),
        }
    }
}

impl FileReader for IOFileReader {
    fn get_text(&self, uuid: uuid::Uuid) -> Option<String> {
        self.files.get(&uuid).map(|(_, text)| text.clone())
    }
    fn get_filename(&self, uuid: uuid::Uuid) -> Option<String> {
        self.files.get(&uuid).map(|(path, _)| path.clone())
    }

    fn import_file(
        &mut self,
        path: &str,
        in_file: Option<uuid::Uuid>,
    ) -> Result<(Uuid, Peekable<Lexer>), FileReaderError> {
        let path = if let Some(id) = in_file {
            // get parent from uuid
            let parent = self.files.get(&id).map(|(path, _)| path);
            if let Some(parent) = parent {
                // join parent path to path
                let parent = PathBuf::from_str(parent)
                    .ok()
                    .ok_or(FileReaderError::InvalidPath)?;
                let parent = parent.parent().ok_or(FileReaderError::InvalidPath)?;
                parent
                    .join(path)
                    .to_str()
                    .ok_or(FileReaderError::InvalidPath)?
                    .to_owned()
            } else {
                return Err(FileReaderError::InternalFileNotFound);
            }
        } else {
            let full_path = PathBuf::from_str(path).map_err(|_| FileReaderError::InvalidPath)?;
            full_path
                .canonicalize()
                .map_err(|_| FileReaderError::Unexpected)?
                .to_str()
                .ok_or(FileReaderError::Unexpected)?
                .to_owned()
        };

        // open file and read it
        let file = match std::fs::read_to_string(path.clone()) {
            Ok(file) => file,
            Err(err) => return Err(FileReaderError::IOErr(err.to_string())),
        };

        // store full path to file
        let uuid = uuid::Uuid::new_v4();
        if self
            .files
            .insert(uuid, (path.clone(), file.clone()))
            .is_some()
        {
            return Err(FileReaderError::FileAlreadyRead(path));
        }

        // create lexer
        let lexer = Lexer::new(file, uuid);

        Ok((uuid, lexer.peekable()))
    }
}

trait ErrorDisplay {
    fn display_errors<T: FileReader + Clone>(&self, parser: &RVParser<T>);
}

impl ErrorDisplay for Vec<DiagnosticItem> {
    fn display_errors<T: FileReader + Clone>(&self, parser: &RVParser<T>) {
        for err in self {
            let filename = parser
                .reader
                .get_filename(err.file)
                .unwrap_or("unknown".to_owned());
            let text = parser.reader.get_text(err.file).unwrap();
            PrettyPrinter::new()
                .input(
                    Input::from_reader(text.as_bytes())
                        .kind("File")
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

            print!(
                "       {}\n       {}\n",
                err.title.bold(),
                err.long_description
            );
        }
    }
}
fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Lint(lint) => {
            let reader = IOFileReader::new();
            let mut parser = RVParser::new(reader);

            let mut diags = Vec::new();
            let parsed = parser.parse(
                lint.input
                    .to_str()
                    .expect("unable to convert path to string"),
                false,
            );
            parsed
                .1
                .iter()
                .for_each(|x| diags.push(DiagnosticItem::from(x.clone())));

            let cfg = match Cfg::new(parsed.0) {
                Ok(cfg) => cfg,
                Err(err) => {
                    diags.push(DiagnosticItem::from(*err));
                    diags.sort();
                    diags.display_errors(&parser);
                    return;
                }
            };

            match Manager::gen_full_cfg(cfg) {
                Ok(full_cfg) => {
                    // if debug, print out the cfg
                    if lint.yaml {
                        let wrapped = riscv_analysis::cfg::CFGWrapper::from(&full_cfg);
                        println!("{}", serde_yaml::to_string(&wrapped).unwrap());
                    } else if lint.debug {
                        println!("{}", full_cfg);
                    }
                    let mut errs = Vec::new();
                    Manager::run_diagnostics(&full_cfg, &mut errs);
                    errs.iter()
                        .for_each(|x| diags.push(DiagnosticItem::from(x.clone())));
                }
                Err(err) => {
                    diags.push(DiagnosticItem::from(*err));
                }
            };

            if !lint.no_output {
                diags.sort();
                diags.display_errors(&parser);
            }
        }
        Commands::Fix(_) => {}
        Commands::DebugParse(debu) => {
            // Debug mode that prints out parsing errors only
            let reader = IOFileReader::new();
            let mut parser = RVParser::new(reader);
            let parsed = parser.parse(
                debu.input
                    .to_str()
                    .expect("unable to convert path to string"),
                true,
            );
            for err in parsed.1 {
                println!(
                    "({}, {}): {}",
                    parser
                        .reader
                        .get_filename(err.file())
                        .unwrap_or("unknown".to_owned()),
                    err.range(),
                    err
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::IOFileReader;
    use riscv_analysis::cfg::CFGWrapper;
    use riscv_analysis::cfg::Cfg;
    use riscv_analysis::parser::RVParser;
    use riscv_analysis::passes::Manager;

    macro_rules! file_name {
        ($fname:expr) => {
            concat!(env!("CARGO_MANIFEST_DIR"), "/resources/test/", $fname) // assumes Linux ('/')!
        };
    }

    macro_rules! file_test_case {
        ($fname:ident) => {
            #[test]
            fn $fname() {
                let filename = concat!(file_name!(stringify!($fname)), "/code.s");
                let compare = concat!(file_name!(stringify!($fname)), "/raw.yaml");
                let reader = IOFileReader::new();
                let mut parser = RVParser::new(reader);

                let parsed = parser.parse(filename, false);

                let cfg = Cfg::new(parsed.0).unwrap();
                let res: Cfg = Manager::gen_full_cfg(cfg).unwrap();
                let res = CFGWrapper::from(&res);

                // deserialize the yaml file
                let compare = std::fs::read_to_string(compare).unwrap();
                let compare: CFGWrapper = serde_yaml::from_str(&compare).unwrap();

                // read the res into yaml format to match
                let res = serde_yaml::to_string(&res).unwrap();
                let res: CFGWrapper = serde_yaml::from_str(&res).unwrap();

                assert_eq!(res, compare);
            }
        };
    }
    file_test_case!(loop_check);
    file_test_case!(treg);
}
