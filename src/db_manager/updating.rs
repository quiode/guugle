use rusqlite::Connection;

use super::creation::DatabaseConnection;

// # sets all in use to false
// used when opening a new database that maybe hasn't been closed correctely
pub fn reset_in_use(conn: &Connection) -> Result<usize, rusqlite::Error> {
    conn.execute("UPDATE Ranking SET in_use = false;", ())
}

/// updates the database entry for the page to visited and fills in the required data
pub fn update_to_visited(
    conn: &DatabaseConnection,
    id: i64,
    content: &str,
    links_to: Vec<&str>,
    verbose: bool,
) -> Result<i64, rusqlite::Error> {
    conn.connection.execute(
        "UPDATE Ranking SET visited = true, content = ?1, links_to = ?2 WHERE id = ?3;",
        [content, &links_to.join(":::"), &id.to_string()],
    )?;

    if verbose {
        println!("Crawled webpage with id: {}", id);
    }

    Ok(id)
}

pub fn set_in_use(conn: &DatabaseConnection, id: i64, state: bool) -> Result<i64, rusqlite::Error> {
    let mut statement = conn
        .connection
        .prepare("UPDATE Ranking SET in_use = ?1 WHERE id = ?2;")?;

    statement.execute((state, id))?;

    Ok(id)
}

#[cfg(test)]
pub mod tests {
    use std::fs;

    use crate::db_manager::{
        creation::create_default_tables,
        helper::{count_rows, gen_random_path, gen_vals},
        updating::{reset_in_use, set_in_use, update_to_visited},
    };

    #[test]
    fn test_update_to_visited() {
        let path = gen_random_path();

        let conn = create_default_tables(path.to_str().unwrap()).unwrap();

        // fill db with values
        let mut prep = conn
            .connection
            .prepare("INSERT INTO Ranking (url, links_to) VALUES (?1, ?2)")
            .unwrap();

        prep.execute(("test.ch", "team-crystal.ch:::google.ch:::example.com"))
            .unwrap();
        prep.execute(("help.ch", "team-crystal.ch:::google.ch:::test.ch"))
            .unwrap();
        prep.execute(("p.ch", "help.ch")).unwrap();
        prep.execute(("ep.ch", "team-crystal.ch:::help.ch"))
            .unwrap();
        prep.execute(("lp.ch", "help.ch:::google.ch")).unwrap();

        let content = r#"
            <!doctype html>
                <html>
                <head>
                    <title>Example Domain</title>

                    <meta charset="utf-8" />
                    <meta http-equiv="Content-type" content="text/html; charset=utf-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1" />
                </head>

                <body>
                <div>
                    <h1>Example Domain</h1>
                    <p>This domain is for use in illustrative examples in documents. You may use this
                    domain in literature without prior coordination or asking for permission.</p>
                    <p><a href="https://www.iana.org/domains/example">More information...</a></p>
                </div>
                </body>
                </html>
        "#;

        let links_to = vec!["ep.ch", "lp.ch"].join(":::");

        // update value
        update_to_visited(&conn, 1, content, vec!["ep.ch", "lp.ch"], false).unwrap();

        // test if values have been updatet
        let row: (i64, String, String) = conn
            .connection
            .prepare("SELECT visited, content, links_to FROM Ranking WHERE id = ?1;")
            .unwrap()
            .query_row(["1"], |row| {
                Ok((row.get_unwrap(0), row.get_unwrap(1), row.get_unwrap(2)))
            })
            .unwrap();

        fs::remove_file(path).unwrap();

        assert_eq!(row.0, 1);
        assert_eq!(row.1, content.to_string());
        assert_eq!(row.2, links_to);
    }

    #[test]
    fn clear_in_use() {
        let path = gen_random_path();

        let conn = create_default_tables(path.to_str().unwrap()).unwrap();

        // fill db with values
        let mut prep = conn
            .connection
            .prepare("INSERT INTO Ranking (url, links_to, in_use) VALUES (?1, ?2, true)")
            .unwrap();

        prep.execute(("test.ch", "team-crystal.ch:::google.ch:::example.com"))
            .unwrap();
        prep.execute(("help.ch", "team-crystal.ch:::google.ch:::test.ch"))
            .unwrap();
        prep.execute(("p.ch", "help.ch")).unwrap();
        prep.execute(("ep.ch", "team-crystal.ch:::help.ch"))
            .unwrap();
        prep.execute(("lp.ch", "help.ch:::google.ch")).unwrap();

        reset_in_use(&conn.connection).unwrap();

        // check if db are correct
        let mut statement = conn
            .connection
            .prepare("SELECT * FROM Ranking WHERE in_use = true;")
            .unwrap();

        let rows = statement.query(());

        let count = count_rows(rows).unwrap();

        fs::remove_file(path).unwrap();

        assert_eq!(count, 0);
    }

    #[test]
    fn set_in_use_test() {
        let path = gen_random_path();

        let conn = create_default_tables(path.to_str().unwrap()).unwrap();

        gen_vals(&conn);

        assert!(!conn
            .connection
            .prepare("SELECT in_use FROM Ranking WHERE id = 3")
            .unwrap()
            .query_row((), |row| Ok(row.get::<usize, bool>(0).unwrap()))
            .unwrap());

        set_in_use(&conn, 3, true).unwrap();

        fs::remove_file(path).unwrap();
        assert!(conn
            .connection
            .prepare("SELECT in_use FROM Ranking WHERE id = 3")
            .unwrap()
            .query_row((), |row| Ok(row.get::<usize, bool>(0).unwrap()))
            .unwrap());
    }
}
