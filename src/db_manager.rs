use rusqlite::Connection;

/// creates the default tables and database if it doens't already exist
fn create_default_tables(sqlite_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open(sqlite_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Ranking (
	id PRIMARY KEY,
  	url TEXT NOT NULL,
  	content TEXT,
  	links_to TEXT,
  	links_from text,
  	page_rank integer);",
        (),
    )?;

    Ok(())
}

/// creates an entry in the database for a newly discovered page
/// returns the id of the entry
fn unvisited_page(url: &str) -> u64 {
    todo!()
}

/// updates the database entry for the page to visited and fills in the required data
/// the pagerank fiel and the from field gets calculated by the function itself
fn update_to_visited(id: u64, text: &str, points_to: Vec<&str>) -> u64 {
    todo!()
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use rusqlite::Connection;

    use super::create_default_tables;

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

    fn gen_random_path() -> PathBuf {
        let path = format!("./{}.db3", uuid::Uuid::new_v4().to_string());

        Path::new(&path).to_owned()
    }
}
