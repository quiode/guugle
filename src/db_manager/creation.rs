use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::indexer::visit_types::ToVisit;

use super::updating::reset_in_use;

#[derive(Debug)]
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
    visited BOOLEAN NOT NULL DEFAULT false CHECK (visited IN (false, true)),
      url TEXT NOT NULL UNIQUE,
      content TEXT,
      links_to TEXT,
    in_use BOOLEAN NOT NULL DEFAULT false CHECK (visited IN (false, true)));",
        (),
    )?;

    reset_in_use(&conn)?;

    Ok(DatabaseConnection {
        connection: conn,
        _private: (),
    })
}

/// creates an entry in the database for a newly discovered page
/// returns a new ToVisit instance
pub fn unvisited_page(
    conn: Arc<Mutex<DatabaseConnection>>,
    url: &str,
) -> Result<ToVisit, rusqlite::Error> {
    let id: i64;

    {
        let statement = conn.lock();
        let statement = statement.unwrap();

        let mut statement = statement
            .connection
            .prepare("INSERT INTO Ranking (url) VALUES (?);")?;

        id = statement.insert([url])?;
    }

    ToVisit::new(url, id, conn)
}

#[cfg(test)]
pub mod tests {
    use std::{
        fs,
        sync::{Arc, Mutex},
    };

    use rusqlite::Connection;

    use crate::db_manager::{
        creation::create_default_tables,
        helper::{gen_random_path, gen_vals},
    };

    use super::unvisited_page;

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

    /// tests unvisited_page
    #[test]
    fn unvisited_created() {
        const WORD: &str = "kampfwort90.ch";
        // prepare database
        let path = gen_random_path();
        let conn = create_default_tables(path.to_str().unwrap()).unwrap();
        gen_vals(&conn);

        let conn = Arc::new(Mutex::new(conn));

        // call function that is tested
        let to_visit = unvisited_page(Arc::clone(&conn), WORD).unwrap();
        let conn = conn.lock().unwrap();
        let mut statement = conn
            .connection
            .prepare("SELECT url FROM Ranking WHERE id = ?1;")
            .unwrap();
        let result = statement
            .query_row([to_visit.id], |r| r.get::<usize, String>(0))
            .unwrap();

        assert_eq!(to_visit.url, WORD);
        drop(to_visit);
        assert_eq!(result, WORD);
        fs::remove_file(path).unwrap();
    }
}
