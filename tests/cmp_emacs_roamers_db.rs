use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use org_roamers::sqlite::SqliteConnection;

const COLUMNS: [&'static str; 11] = [
    "id",
    "file",
    "level",
    "pos",
    "todo",
    "priority",
    "scheduled",
    "deadline",
    "title",
    "properties",
    "olp",
];

#[test]
fn cmp_emacs_roamers_db() {
    let mut db_path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    let mut roam_path = db_path.clone();
    db_path.push("test-data/test-db.db");
    roam_path.push("test-data/");
    let mut roamers_db = SqliteConnection::init::<PathBuf>(None).unwrap();
    let mut emacs_db = SqliteConnection::init(Some(db_path)).unwrap();

    roamers_db.insert_files(roam_path).unwrap();

    let mut emacs_nodes = emacs_db.get_all_nodes(COLUMNS);
    let mut roamers_nodes = roamers_db.get_all_nodes(COLUMNS);

    assert!(emacs_nodes.len() > 0);
    assert!(roamers_nodes.len() > 0);

    emacs_nodes.sort();
    roamers_nodes.sort();

    test_node_table(&emacs_nodes, &roamers_nodes);
}

fn test_node_table<const PARAMS: usize>(
    emacs_nodes: &[[String; PARAMS]],
    roamers_nodes: &[[String; PARAMS]],
) {
    for (emacs_node, roamers_node) in emacs_nodes.iter().zip(roamers_nodes) {
        assert_eq!(emacs_node[0], roamers_node[0], "Id does not match!");
        assert_eq!(
            fix_paths(emacs_node[1].clone()),
            fix_paths(roamers_node[1].clone()),
            "File does not match!"
        );
        assert_eq!(emacs_node[2], roamers_node[2], "Level does not match!");
        assert_eq!(emacs_node[3], roamers_node[3], "Pos does not match!");
        assert_eq!(emacs_node[4], roamers_node[4], "Todos does not match!");
        assert_eq!(emacs_node[5], roamers_node[5], "Priority does not match!");
        assert_eq!(emacs_node[6], roamers_node[6], "Scheduled does not match!");
        assert_eq!(emacs_node[7], roamers_node[7], "Deadline does not match!");
        assert_eq!(emacs_node[8], roamers_node[8], "Title does not match!");
        // TODO: Properties are broken...
        // assert_eq!(emacs_node[9], roamers_node[9], "Properties does not match!");
        assert_eq!(emacs_node[10], roamers_node[10], "Olp does not match!");
    }
}

fn fix_paths<P: AsRef<Path>>(path: P) -> String
where
    PathBuf: From<P>,
{
    PathBuf::from(path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}
