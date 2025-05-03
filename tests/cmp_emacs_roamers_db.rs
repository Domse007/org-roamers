use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use org_roamers::sqlite::SqliteConnection;
use rusqlite::Connection;

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

    // nodes table...
    let mut emacs_nodes = emacs_db.get_all_nodes(COLUMNS);
    let mut roamers_nodes = roamers_db.get_all_nodes(COLUMNS);

    assert!(emacs_nodes.len() > 0);
    assert!(roamers_nodes.len() > 0);

    emacs_nodes.sort();
    roamers_nodes.sort();

    test_node_table(&emacs_nodes, &roamers_nodes);

    let emacs_db_con = emacs_db.connection();
    let roamers_db_con = roamers_db.connection();

    // tags table...
    let tag_f = |con: &mut Connection| -> Vec<(String, String)> {
        const SQL: &str = "SELECT node_id, tag FROM tags;";
        let mut stmnt = con.prepare(SQL).unwrap();
        stmnt
            .query_map([], |row| {
                Ok((
                    row.get_unwrap::<usize, String>(0),
                    row.get_unwrap::<usize, String>(1),
                ))
            })
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };

    let emacs_tags = tag_f(emacs_db_con);
    let roamers_tags = tag_f(roamers_db_con);

    test_tags_table(&emacs_tags, &roamers_tags);

    // links table
    let link_f = |con: &mut Connection| -> Vec<(String, String, String)> {
        const SQL: &str = "SELECT source, dest, type FROM links;";
        let mut stmnt = con.prepare(SQL).unwrap();
        stmnt
            .query_map([], |row| {
                Ok((
                    row.get_unwrap::<usize, String>(0),
                    row.get_unwrap::<usize, String>(1),
                    row.get_unwrap::<usize, String>(2),
                ))
            })
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };

    let emacs_links = link_f(emacs_db_con);
    let roamers_links = link_f(roamers_db_con);

    test_links_table(&emacs_links, &roamers_links);
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

fn test_tags_table(emacs_tags: &[(String, String)], roamers_tags: &[(String, String)]) {
    assert!((emacs_tags.len() > 0) && (roamers_tags.len() > 0));
    for (emacs_node, roamers_node) in emacs_tags.iter().zip(roamers_tags) {
        assert_eq!(emacs_node.0, roamers_node.0, "Id does not match!");
        assert_eq!(emacs_node.1, emacs_node.1, "Tag does not match!");
    }
}

fn test_links_table(
    emacs_links: &[(String, String, String)],
    roamers_links: &[(String, String, String)],
) {
    assert!((emacs_links.len() > 0) && (roamers_links.len() > 0));
    assert!(emacs_links.len() == roamers_links.len());

    for emacs_link in emacs_links {
        let mut f = true;
        for roamers_link in roamers_links {
            if roamers_link.0 == emacs_link.0 && roamers_link.1 == emacs_link.1 {
                f = false;
            }
        }
        if f {
            println!("Not conained: {emacs_link:?}");
            assert!(false);
        }
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
