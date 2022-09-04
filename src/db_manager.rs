use rusqlite::{Connection, Result, Row, Rows};

use crate::indexer::ToVisit;

#[readonly::make]
pub struct DatabaseConnection {
    #[readonly]
    pub connection: Connection,
    _private: (),
}

/// creates the default tables and database if it doens't already exist
/// returns a connection on which all opperations should be worked on
pub fn create_default_tables(
    sqlite_path: &str,
) -> Result<DatabaseConnection, Box<dyn std::error::Error>> {
    let conn = Connection::open(sqlite_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Ranking (
	id INTEGER NOT NULL PRIMARY KEY,
    visited BOOLEAN NOT NULL DEFAULT false CHECK (visited IN (0, 1)),
  	url TEXT NOT NULL UNIQUE,
  	content TEXT,
  	links_to TEXT,
    in_use BOOLEAN NOT NULL DEFAULT false CHECK (visited IN (0, 1)));",
        (),
    )?;

    reset_in_use(&conn);

    Ok(DatabaseConnection {
        connection: conn,
        _private: (),
    })
}

// # sets all in use to false
// used when opening a new database that maybe hasn't been closed correctely
fn reset_in_use(conn: &Connection) -> Result<usize, rusqlite::Error> {
    conn.execute("UPDATE Ranking SET in_use = false;", ())
}

/// creates an entry in the database for a newly discovered page
/// returns a new ToVisit instance
pub fn unvisited_page(conn: &DatabaseConnection, url: &str) -> Result<ToVisit, rusqlite::Error> {
    let mut statement = conn
        .connection
        .prepare("INSERT INTO Ranking (url) VALUES (?);")?;

    let id = statement.insert([url])?;

    Ok(ToVisit::new(url, id))
}

// returns true if all links have been visited
pub fn is_finished(conn: &DatabaseConnection) -> Result<bool, rusqlite::Error> {
    let mut statement = conn
        .connection
        .prepare("SELECT COUNT(*) FROM Ranking WHERE visited = true;")?;

    let result: i64 = statement.query_row((), |row| row.get(0)).unwrap();

    Ok(result == 0)
}

/// updates the database entry for the page to visited and fills in the required data
pub fn update_to_visited(
    conn: &DatabaseConnection,
    id: i64,
    content: &str,
    links_to: Vec<&str>,
) -> Result<i64, rusqlite::Error> {
    conn.connection.execute(
        "UPDATE Ranking SET visited = true, content = ?1, links_to = ?2 WHERE id = ?3;",
        [content, &links_to.join(":::"), &id.to_string()],
    )?;

    Ok(id)
}

/// calculates how many pages point to this page
fn calculate_links_from(conn: &DatabaseConnection, id: i64) -> Result<usize, rusqlite::Error> {
    let url: String = conn
        .connection
        .prepare("SELECT url FROM Ranking WHERE id = ?1;")?
        .query_row([id], |row| row.get(0))?;

    let mut statement = conn
        .connection
        .prepare(format!("SELECT * FROM Ranking WHERE links_to LIKE '%{}%';", url).as_str())?;

    count_rows(statement.query(()))
}

// counts how many rows the sql select statement outputed
fn count_rows(rows: Result<Rows<'_>>) -> Result<usize, rusqlite::Error> {
    let values: Vec<_> = rows?.mapped(|_| Ok(())).collect();

    Ok(values.len())
}

// returns a new link that can be searched if new links exist
pub fn get_new_link(conn: &DatabaseConnection) -> Option<ToVisit> {
    let mut statement = conn
        .connection
        .prepare("SELECT id, url FROM Ranking WHERE in_use = false AND visited = false LIMIT 1;")
        .ok()?;

    let result: (i64, String) = statement
        .query_row((), |row| Ok((row.get(0).unwrap(), row.get(1).unwrap())))
        .ok()?;

    Some(ToVisit::new(&result.1, result.0))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use rusqlite::Connection;

    use super::*;

    #[test]
    fn file_created() {
        let path = gen_random_path();

        let test_result = create_default_tables(path.to_str().unwrap());

        assert!(path.exists());
        fs::remove_file(path).unwrap();

        assert!(test_result.is_ok());
    }

    #[test]
    fn table_created() {
        let path = gen_random_path();

        create_default_tables(path.to_str().unwrap()).unwrap();

        let conn = Connection::open(path.to_str().unwrap()).unwrap();

        let result = conn.execute("SELECT * FROM Ranking;", ());

        fs::remove_file(path).unwrap();

        result.unwrap();
    }

    #[test]
    fn calculates_links() {
        let path = gen_random_path();

        let conn = create_default_tables(path.to_str().unwrap()).unwrap();

        // fill db with values
        let mut prep = conn
            .connection
            .prepare("INSERT INTO Ranking (url, links_to) VALUES (?1, ?2)")
            .unwrap();

        prep.execute(["test.ch", "team-crystal.ch:::google.ch:::example.com"])
            .unwrap();
        prep.execute(["help.ch", "team-crystal.ch:::google.ch:::test.ch"])
            .unwrap();
        prep.execute(["lp.ch", "help.ch"]).unwrap();
        prep.execute(["ep.ch", "team-crystal.ch:::help.ch"])
            .unwrap();
        prep.execute(["lp.ch", "help.ch:::google.ch"]).unwrap();

        let test_result1 = calculate_links_from(&conn, 2).unwrap();

        let test_result2 = calculate_links_from(&conn, 1).unwrap();

        fs::remove_file(path).unwrap();

        assert_eq!(test_result1, 3);

        assert_eq!(test_result2, 1);
    }

    #[test]
    fn test_update_to_visited() {
        let path = gen_random_path();

        let conn = create_default_tables(path.to_str().unwrap()).unwrap();

        // fill db with values
        let mut prep = conn
            .connection
            .prepare("INSERT INTO Ranking (url, links_to) VALUES (?1, ?2)")
            .unwrap();

        prep.execute(["test.ch", "team-crystal.ch:::google.ch:::example.com"])
            .unwrap();
        prep.execute(["help.ch", "team-crystal.ch:::google.ch:::test.ch"])
            .unwrap();
        prep.execute(["lp.ch", "help.ch"]).unwrap();
        prep.execute(["ep.ch", "team-crystal.ch:::help.ch"])
            .unwrap();
        prep.execute(["lp.ch", "help.ch:::google.ch"]).unwrap();

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
        update_to_visited(&conn, 1, content, vec!["ep.ch", "lp.ch"]).unwrap();

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

        prep.execute(["test.ch", "team-crystal.ch:::google.ch:::example.com"])
            .unwrap();
        prep.execute(["help.ch", "team-crystal.ch:::google.ch:::test.ch"])
            .unwrap();
        prep.execute(["lp.ch", "help.ch"]).unwrap();
        prep.execute(["ep.ch", "team-crystal.ch:::help.ch"])
            .unwrap();
        prep.execute(["lp.ch", "help.ch:::google.ch"]).unwrap();

        // check if db are correct
        let mut statement = conn
            .connection
            .prepare("SELECT * FROM Ranking WHERE in_use = true;")
            .unwrap();

        let rows = statement.query(());

        let count = count_rows(rows).unwrap();

        fs::remove_file(path).unwrap();

        assert_eq!(count, 5);
    }

    #[test]
    fn get_new_link_test() {
        let path = gen_random_path();

        let conn = create_default_tables(path.to_str().unwrap()).unwrap();

        // fill db with values
        let mut prep = conn
            .connection
            .prepare("INSERT INTO Ranking (url, links_to, in_use, visited) VALUES (?1, ?2, ?3, ?4)")
            .unwrap();

        prep.execute([
            "test.ch",
            "team-crystal.ch:::google.ch:::example.com",
            "true",
            "false",
        ])
        .unwrap();
        prep.execute([
            "help.ch",
            "team-crystal.ch:::google.ch:::test.ch",
            "false",
            "false",
        ])
        .unwrap();
        prep.execute(["lp.ch", "help.ch", "false", "true"]).unwrap();
        prep.execute(["ep.ch", "team-crystal.ch:::help.ch", "false, false"])
            .unwrap();
        prep.execute(["lp.ch", "help.ch:::google.ch", "true", "true"])
            .unwrap();

        // check if desired value is returned
        let link = get_new_link(&conn).unwrap();

        assert_eq!(link.url, "help.ch");
    }

    #[test]
    fn is_finished_false() {
        let path = gen_random_path();

        let conn = create_default_tables(path.to_str().unwrap()).unwrap();

        // fill db with values
        let mut prep = conn
            .connection
            .prepare("INSERT INTO Ranking (url, links_to, in_use, visited) VALUES (?1, ?2, ?3, ?4)")
            .unwrap();

        prep.execute([
            "test.ch",
            "team-crystal.ch:::google.ch:::example.com",
            "true",
            "false",
        ])
        .unwrap();
        prep.execute([
            "help.ch",
            "team-crystal.ch:::google.ch:::test.ch",
            "false",
            "false",
        ])
        .unwrap();
        prep.execute(["lp.ch", "help.ch", "false", "true"]).unwrap();
        prep.execute(["ep.ch", "team-crystal.ch:::help.ch", "false, false"])
            .unwrap();
        prep.execute(["lp.ch", "help.ch:::google.ch", "true", "true"])
            .unwrap();

        let result = is_finished(&conn).unwrap();

        assert!(!result)
    }

    fn is_finished_true() {
        let path = gen_random_path();

        let conn = create_default_tables(path.to_str().unwrap()).unwrap();

        // fill db with values
        let mut prep = conn
            .connection
            .prepare("INSERT INTO Ranking (url, links_to, in_use, visited) VALUES (?1, ?2, ?3, ?4)")
            .unwrap();

        prep.execute([
            "test.ch",
            "team-crystal.ch:::google.ch:::example.com",
            "false",
            "true",
        ])
        .unwrap();
        prep.execute([
            "help.ch",
            "team-crystal.ch:::google.ch:::test.ch",
            "false",
            "true",
        ])
        .unwrap();
        prep.execute(["lp.ch", "help.ch", "false", "true"]).unwrap();
        prep.execute(["ep.ch", "team-crystal.ch:::help.ch", "false, true"])
            .unwrap();
        prep.execute(["lp.ch", "help.ch:::google.ch", "false", "true"])
            .unwrap();

        let result = is_finished(&conn).unwrap();

        assert!(result)
    }

    fn gen_random_path() -> PathBuf {
        let path = format!("./{}.db3", uuid::Uuid::new_v4().to_string());

        Path::new(&path).to_owned()
    }
}
