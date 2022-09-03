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
  	url TEXT NOT NULL,
  	content TEXT,
  	links_to TEXT,
  	page_rank integer);",
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
/// the pagerank fiel and the from field gets calculated by the function itself
fn update_to_visited(
    conn: &DatabaseConnection,
    id: i64,
    content: &str,
    links_to: Vec<&str>,
) -> Result<i64, rusqlite::Error> {
    let mut statement = conn
        .connection
        .prepare("UPDATE Ranking SET content = ?1, links_to = ?2, page_rank = ?3 WHERE id = ?4;")?;

    todo!()
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

    fn gen_random_path() -> PathBuf {
        let path = format!("./{}.db3", uuid::Uuid::new_v4().to_string());

        Path::new(&path).to_owned()
    }
}
