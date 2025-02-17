use std::{env, process::ExitCode};

use emacs::Env;
use org_roam_rs::{add_node, init_db, Logger, StdOutLogger};
use orgize::Org;
use std::fs::{self, DirEntry, FileType};
use std::path::Path;

fn main() -> ExitCode {
    let path = env::var("DB").unwrap();
    let logger = StdOutLogger;

    (&logger)
        .log(format!("Using {path} for indexing."))
        .unwrap();

    // init like emacs would.
    init_db(&logger).unwrap();

    (&logger)
        .log("Successfully initalized the logger.")
        .unwrap();

    //add_files(path, &logger);

    ExitCode::SUCCESS
}

fn add_files<P: AsRef<Path>>(path: P, logger: impl Logger + Copy) {
    let mut iter = fs::read_dir(path.as_ref()).unwrap();
    while let Some(entry) = iter.next() {
        match entry {
            Ok(e) => process_entry(e, logger),
            err => logger.log(format!("{err:?}")).unwrap(),
        }
    }
}

fn process_entry(entry: DirEntry, logger: impl Logger + Copy) {
    let t = entry.file_type().unwrap();

    logger.log(format!("Processing {entry:?} {t:?}"));

    if t.is_dir() {
        return add_files(entry.path(), logger);
    }

    if t.is_file() && entry.path().extension().unwrap() == "org" {
        let file = fs::read_to_string(entry.path()).unwrap();
        let ast = Org::parse_string(file);

        let id = ast.document().section_node().unwrap();
        let elem = ast.document().children(&ast).next().unwrap();
        let title = &elem.title(&ast).raw;

        // add_node(title.to_string(), id.to_string(), String::default()).unwrap();

        return;
    }

    logger
        .log("Could not process thing: {t} :: {entry}")
        .unwrap();
}
