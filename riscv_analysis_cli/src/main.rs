use std::fmt::Display;
use std::io::Write;
use std::vec;
use std::{collections::HashMap, iter::Peekable, str::FromStr};

use bat::line_range::{LineRange, LineRanges};
use bat::{Input, PrettyPrinter};
use colored::Colorize;
use riscv_analysis::fix::{fix_stack, Manipulation};
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

#[derive(Copy, Clone)]
enum OffsetItem {
    /// Add (1) chars and (2) lines at index (0)
    ///
    /// When this is in play, any line after or including (0) idx
    /// has (1) added to it.
    Add(usize, usize, usize),
    /// Remove (1) chars and (2) lines at index (0)
    ///
    /// When this is in play, any char after the end of the range
    /// is removed. No offset should ever be in the range, so the
    /// program will panic if dones so.
    Remove(usize, usize, usize),
}

/// Offset adjustments when changes are made
///
/// All offset items in the list are relative to the original
/// offset at the beginning.
#[derive(Clone)]
struct OffsetList(Vec<OffsetItem>);

impl OffsetList {
    fn new() -> Self {
        OffsetList(Vec::new())
    }
    fn add_change(&mut self, item: OffsetItem) {
        self.0.push(item);
    }

    fn real_offset(&self, mut idx: usize, mut line_orig: usize) -> (usize, usize) {
        for off in self.0.iter() {
            match off {
                OffsetItem::Add(id, chars, line) => {
                    if idx >= *id {
                        idx += chars;
                    }
                    if line_orig >= *line {
                        line_orig += line;
                    }
                }
                OffsetItem::Remove(id, chars, line) => {
                    if idx >= *id && idx < *id + *chars {
                        idx = *id
                    } else if idx >= *id {
                        idx -= chars;
                    }
                    if line_orig >= *line {
                        line_orig -= line;
                    }
                }
            }
        }
        (idx, line_orig)
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
        let mut changed_files: HashMap<uuid::Uuid, (String, String, OffsetList)> = HashMap::new();
        let mut changed_ranges = Vec::new();

        for fix in fixes {
            // check if we already have changed this file
            // otherwise, get the file details
            let (path, source, mut offset_list) = if let Some(x) = changed_files.get(&fix.file()) {
                (x.0.clone(), x.1.clone(), x.2.clone())
            } else {
                let file_details = self
                    .files
                    .get(&fix.file())
                    .ok_or(ManipulationError::InternalError)?;
                let res = file_details.clone();
                // changed_files.insert(fix.file(), (res.0.clone(), res.1.clone(), 0));
                (res.0, res.1, OffsetList::new())
            };

            let row = fix.line();
            let pos = fix.raw_pos() - 1;

            let file = fix.file();

            // insert fix text into source
            // TODO need to make this work when things change
            match fix {
                Manipulation::Insert(_, _, s, lines) => {
                    let mut new_source = source.clone();
                    dbg!(&new_source);
                    let (pos, row) = offset_list.real_offset(pos, row);
                    new_source.insert_str(pos, &s);
                    dbg!(&new_source);
                    changed_ranges.push(ChangedRanges {
                        filename: path.clone(),
                        file,
                        begin_window: row - 2,
                        end_window: row + lines as usize + 1,
                        begin_change: row,
                        end_change: row + lines as usize - 1,
                    });
                    offset_list.add_change(OffsetItem::Add(pos, s.len(), lines));
                    changed_files.insert(file, (path.clone(), new_source, offset_list));
                }
                Manipulation::Replace(_, range, s, lines_removed, lines_added) => {
                    let mut new_source = source.clone();
                    let len = range.end.raw_index - range.start.raw_index;
                    let (pos, _) = offset_list.real_offset(pos, row);
                    new_source.drain(std::ops::Range {
                        start: pos as usize,
                        end: pos + len as usize,
                    });
                    offset_list.add_change(OffsetItem::Remove(pos, len, lines_removed));
                    let (pos, row) = offset_list.real_offset(pos, row);
                    new_source.insert_str((range.start.raw_index as isize - 1) as usize, &s);

                    changed_ranges.push(ChangedRanges {
                        filename: path.clone(),
                        file,
                        begin_window: row - 2,
                        end_window: row + lines_added - lines_removed + 1,
                        begin_change: row,
                        end_change: row + lines_added - lines_removed as usize - 1,
                    });
                    offset_list.add_change(OffsetItem::Add(pos, s.len(), lines_added));
                    changed_files.insert(file, (path.clone(), new_source, offset_list));
                }
            }
        }

        // display changed ranges
        // FIXME we just show the whole file to be easier for now, need to make it more specific in
        // the future.
        // for changed_range in changed_ranges {
        //     let input =
        //         Input::from_reader(changed_files.get(&changed_range.file).unwrap().1.as_bytes())
        //             .name(changed_range.filename.clone());
        //     PrettyPrinter::new()
        //         .input(input)
        //         .line_numbers(true)
        //         .header(true)
        //         .grid(true)
        //         .line_ranges(LineRanges::from(vec![LineRange::new(
        //             changed_range.begin_window + 1,
        //             changed_range.end_window + 1,
        //         )]))
        //         .highlight_range(changed_range.begin_change + 1, changed_range.end_change + 1)
        //         .print()
        //         .unwrap();
        // }
        let inputs = changed_files
            .iter()
            .map(|x| Input::from_reader(x.1 .1.as_bytes()).name(x.1 .0.clone()));

        for x in inputs {
            PrettyPrinter::new()
                .input(x)
                .line_numbers(true)
                .header(true)
                .grid(true)
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
            for (path, source, _) in changed_files.values() {
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

            match Manager::gen_full_cfg(parsed.0) {
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
        Commands::Fix(fix) => {
            let reader = IOFileReader::new();
            let mut parser = RVParser::new(reader);
            let parsed = parser.parse(
                fix.input
                    .to_str()
                    .expect("unable to convert path to string"),
                false,
            );
            let cfg = Manager::gen_full_cfg(parsed.0).expect("unable to generate full cfg");

            let func = cfg
                .label_function_map
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

                let res: Cfg = Manager::gen_full_cfg(parsed.0).unwrap();
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
