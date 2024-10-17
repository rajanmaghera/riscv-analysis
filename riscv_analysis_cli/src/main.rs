mod printer;
use printer::*;

use std::fmt::Display;
use std::io::Write;
use std::vec;
use std::{collections::HashMap, str::FromStr};

use bat::line_range::{LineRange, LineRanges};
use bat::{Input, PrettyPrinter};
use colored::Colorize;
use riscv_analysis::fix::{fix_stack, Manipulation};
use riscv_analysis::parser::{Info, LabelString, RVParser, With};
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

#[derive(Debug)]
enum ManipulationError {
    InternalError,
}

impl Display for ManipulationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManipulationError::InternalError => write!(f, "Internal error"),
        }
    }
}

impl IOFileReader {
    fn new() -> Self {
        IOFileReader {
            files: HashMap::new(),
        }
    }
    fn apply_fixes(&self, fixes: Vec<Manipulation>) -> Result<(), ManipulationError> {
        struct ChangedRanges {
            filename: String,
            file: uuid::Uuid,
            begin_window: usize,
            end_window: usize,
            begin_change: usize,
            end_change: usize,
        }

        // map of file uuid to (path, source, offet pos, offset lines)
        let mut changed_files: HashMap<uuid::Uuid, (String, String, i64, i64)> = HashMap::new();
        let mut changed_ranges = Vec::new();

        for fix in fixes {
            // check if we already have changed this file
            // otherwise, get the file details
            let (path, source, mut offset, mut offset_lines) =
                if let Some(x) = changed_files.get(&fix.file()) {
                    (x.0.clone(), x.1.clone(), x.2, x.3)
                } else {
                    let file_details = self
                        .files
                        .get(&fix.file())
                        .ok_or(ManipulationError::InternalError)?;
                    let res = file_details.clone();
                    // changed_files.insert(fix.file(), (res.0.clone(), res.1.clone(), 0));
                    (res.0, res.1, 0, 0)
                };

            let row = fix.line();
            let pos = fix.raw_pos() - 1;

            let file = fix.file();
            // insert fix text into source
            match fix {
                Manipulation::Insert(_, _, s, lines) => {
                    let mut new_source = source.clone();
                    // we know that the insert only inserts, so we can be safe returning the offset
                    // as usize
                    new_source.insert_str(pos + offset as usize, &s);
                    changed_ranges.push(ChangedRanges {
                        filename: path.clone(),
                        file,
                        begin_window: row - 2 + offset_lines as usize,
                        end_window: row + lines + offset_lines as usize + 1,
                        begin_change: row + offset_lines as usize,
                        end_change: row + lines + offset_lines as usize - 1,
                    });
                    offset += s.len() as i64;
                    offset_lines += lines as i64;
                    changed_files.insert(file, (path.clone(), new_source, offset, offset_lines));
                }
            }
        }

        // display changed ranges
        for changed_range in changed_ranges {
            let input =
                Input::from_reader(changed_files.get(&changed_range.file).unwrap().1.as_bytes())
                    .name(changed_range.filename.clone());
            PrettyPrinter::new()
                .input(input)
                .line_numbers(true)
                .header(true)
                .grid(true)
                .line_ranges(LineRanges::from(vec![LineRange::new(
                    changed_range.begin_window + 1,
                    changed_range.end_window + 1,
                )]))
                .highlight_range(changed_range.begin_change + 1, changed_range.end_change + 1)
                .print()
                .unwrap();
        }

        // ask user to apply changes
        let mut apply_changes = false;
        loop {
            let mut input = String::new();
            print!("Apply changes? [y/n] ");
            // flush stdout
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut input).unwrap();
            if input.trim() == "y" {
                apply_changes = true;
                break;
            } else if input.trim() == "n" {
                break;
            }
        }

        if apply_changes {
            for (path, source, _, _) in changed_files.values() {
                let mut file = std::fs::File::create(path).unwrap();
                file.write_all(source.as_bytes()).unwrap();
            }

            println!("{}", "Changes applied.".green());
            println!(
                "{}Please remove all other instances of stack manipulation in your code. This fix did not remove any lines of code.",
                "WARNING: ".red().bold()
            );
        }
        Ok(())
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
        parent_file: Option<uuid::Uuid>,
    ) -> Result<(Uuid, String), FileReaderError> {
        let path = if let Some(id) = parent_file {
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

        Ok((uuid, file))
    }
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Lint(lint) => {
            let reader = IOFileReader::new();
            let mut parser = RVParser::new(reader);

            let mut diags = Vec::new();
            let parsed = parser.parse_from_file(
                lint.input
                    .to_str()
                    .expect("unable to convert path to string"),
                false,
            );
            parsed
                .1
                .iter()
                .for_each(|x| diags.push(DiagnosticItem::from(x.clone())));

            match Manager::gen_full_cfg(parsed.0) {
                Ok(full_cfg) => {
                    // if debug, print out the cfg
                    if lint.yaml {
                        let wrapped = riscv_analysis::cfg::CfgWrapper::from(&full_cfg);
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
                let printer = PrettyPrint::new(diags);
                printer.display_errors(&parser);
            }
        }
        Commands::Fix(fix) => {
            let reader = IOFileReader::new();
            let mut parser = RVParser::new(reader);
            let parsed = parser.parse_from_file(
                fix.input
                    .to_str()
                    .expect("unable to convert path to string"),
                false,
            );
            let cfg = Manager::gen_full_cfg(parsed.0).expect("unable to generate full cfg");

            let funcs = cfg.functions();
            let func = funcs
                .get(&With::new(
                    LabelString(fix.func_name.clone()),
                    Info::default(),
                ))
                .expect("unable to find function");
            let fixes = fix_stack(func);
            parser.reader.apply_fixes(fixes).unwrap();
        }
        Commands::DebugParse(debu) => {
            // Debug mode that prints out parsing errors only
            let reader = IOFileReader::new();
            let mut parser = RVParser::new(reader);
            let parsed = parser.parse_from_file(
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
    use riscv_analysis::cfg::Cfg;
    use riscv_analysis::cfg::CfgWrapper;
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

                let parsed = parser.parse_from_file(filename, false);

                let res: Cfg = Manager::gen_full_cfg(parsed.0).unwrap();
                let res = CfgWrapper::from(&res);

                // deserialize the yaml file
                let compare = std::fs::read_to_string(compare).unwrap();
                let compare: CfgWrapper = serde_yaml::from_str(&compare).unwrap();

                // read the res into yaml format to match
                let res = serde_yaml::to_string(&res).unwrap();
                let res: CfgWrapper = serde_yaml::from_str(&res).unwrap();

                assert_eq!(res, compare);
            }
        };
    }
    file_test_case!(loop_check);
    file_test_case!(treg);
}
