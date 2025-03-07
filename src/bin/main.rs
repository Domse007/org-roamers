use std::{env, process::ExitCode};

use org_roamers::database::Database;
use org_roamers::{init_tantivy, log, Logger, StdOutLogger};
use orgize::Org;
use std::fs::{self, DirEntry};
use std::path::Path;
use anyhow::Result;

use std::hash::Hash;
fn hash<T: Hash>(t: &T) -> u64 {
    use std::hash::{DefaultHasher, Hasher};
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn main() -> Result<ExitCode> {
    let logger = StdOutLogger;

    let cont = org_roamers::org::get_nodes_from_file("test-data/emacs-overview.org")?;
    let db = Database::new()?;
    for node in cont {
        println!("{:?}", node);
        let _ = db.insert_node_deep(node, String::new())?;
    }

    // init like emacs would.
    init_tantivy(&logger, None).unwrap();

    log!(logger, "Successfully initalized the logger.");

    println!(" - - -- -  -- -  -");

    //  9f8a7c1b-4d23-4d9e-ae9b-6f5e3c6b9e9f
    let id = "9f8a7c1b-4d23-4d9e-ae9b-6f5e3c6b9e9f";
    let key = hash(&id);
    let node = db.get_node(key)?;
    println!("FOUND NODE: {:?}", node);

    //add_files(path, &logger);

    Ok(ExitCode::SUCCESS)
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

    log!(logger, "Processing {:?} {:?}", entry, t);

    if t.is_dir() {
        return add_files(entry.path(), logger);
    }

    if t.is_file() && entry.path().extension().unwrap() == "org" {
        // let file = fs::read_to_string(entry.path()).unwrap();
        // let ast = Org::parse(file);

        // let id = ast.document().section_node().unwrap();
        // let elem = ast.document().children(&ast).next().unwrap();
        // let title = &elem.title(&ast).raw;

        // // add_node(title.to_string(), id.to_string(), String::default()).unwrap();

        // return;
    }

    log!(logger, "Could not process thing: {:?} :: {:?}", t, entry);
}
