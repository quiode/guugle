use rusqlite::{Connection, Row};

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
  	url TEXT NOT NULL,
  	content TEXT,
  	links_to TEXT);",
        (),
    )?;

    Ok(DatabaseConnection {
        connection: conn,
        _private: (),
    })
}

/// creates an entry in the database for a newly discovered page
/// returns the id of the entry
fn unvisited_page(conn: &DatabaseConnection, url: &str) -> Result<i64, rusqlite::Error> {
    let mut statement = conn
        .connection
        .prepare("INSERT INTO Ranking (url) VALUES (?);")?;

    statement.insert([url])
}

/// updates the database entry for the page to visited and fills in the required data
fn update_to_visited(
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

    let values: Vec<_> = statement.query([])?.mapped(|_| Ok(())).collect();

    Ok(values.len())
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

    fn gen_random_path() -> PathBuf {
        let path = format!("./{}.db3", uuid::Uuid::new_v4().to_string());

        Path::new(&path).to_owned()
    }
}
