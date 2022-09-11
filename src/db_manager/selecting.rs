use std::sync::{Arc, Mutex};

use crate::indexer::visit_types::ToVisit;

use super::{creation::DatabaseConnection, helper::count_rows, ranking::Ranking};

// returns true if all links have been visited
pub fn is_finished(conn: &DatabaseConnection) -> Result<bool, rusqlite::Error> {
    let mut statement = conn
        .connection
        .prepare("SELECT COUNT(*) FROM Ranking WHERE visited = false;")?;

    let result: i64 = statement.query_row((), |row| row.get(0)).unwrap();

    Ok(result == 0)
}

/// calculates how many pages point to this page
pub fn calculate_links_from(conn: &DatabaseConnection, id: i64) -> Result<usize, rusqlite::Error> {
    let url: String = conn
        .connection
        .prepare("SELECT url FROM Ranking WHERE id = ?1;")?
        .query_row([id], |row| row.get(0))?;

    let mut statement = conn
        .connection
        .prepare(format!("SELECT * FROM Ranking WHERE links_to LIKE '%{}%';", url).as_str())?;

    count_rows(statement.query(()))
}

// returns a new link that can be searched if new links exist
pub fn get_new_link(conn: Arc<Mutex<DatabaseConnection>>) -> Option<ToVisit> {
    let result: (i64, String);

    {
        let lock = conn.lock().unwrap();
        let mut statement = lock
            .connection
            .prepare(
                "SELECT id, url FROM Ranking WHERE in_use = false AND visited = false LIMIT 1;",
            )
            .ok()?;

        result = statement
            .query_row((), |row| Ok((row.get(0).unwrap(), row.get(1).unwrap())))
            .ok()?;
    }

    ToVisit::new(&result.1, result.0, conn).ok()
}

// returns the values stored in the database
pub fn get_values(conn: &DatabaseConnection) -> Result<Vec<Ranking>, rusqlite::Error> {
    let mut statement = conn.connection.prepare("SELECT * FROM Ranking;")?;

    let results = statement.query_map((), |row| {
        Ok(Ranking {
            id: row.get(0)?,
            visited: row.get::<usize, i64>(1)? == 1,
            url: row.get(2)?,
            content: row.get(3)?,
            links_to: row.get(4)?,
            in_use: row.get::<usize, i64>(5)? == 1,
        })
    })?;

    let mut output: Vec<Ranking> = vec![];

    for result in results {
        output.push(result?);
    }

    Ok(output)
}

/// finds all database entries witch include the search value in the url or the content
pub fn find(
    conn: &DatabaseConnection,
    search_value: &str,
) -> Result<Vec<Ranking>, rusqlite::Error> {
    let keywords: Vec<&str> = search_value.split(" ").collect();
    let mut url_search_statement = String::new();
    let mut content_search_statement = String::new();

    for keyword in keywords {
        url_search_statement.push_str(&format!(" url LIKE '%{keyword}%' OR "));
        content_search_statement.push_str(&format!(" content LIKE '%{keyword}%' OR "));
    }

    url_search_statement.push_str(" FALSE ");
    content_search_statement.push_str(" FALSE ");

    let mut statement = conn.connection.prepare(&format!(
        "SELECT * FROM Ranking WHERE {url_search_statement} OR {content_search_statement};"
    ))?;

    let results = statement.query_map((), |row| {
        Ok(Ranking {
            id: row.get(0)?,
            visited: row.get::<usize, i64>(1)? == 1,
            url: row.get(2)?,
            content: row.get(3)?,
            links_to: row.get(4)?,
            in_use: row.get::<usize, i64>(5)? == 1,
        })
    })?;

    let mut output = vec![];

    for result in results {
        output.push(result?);
    }

    Ok(output)
}

#[cfg(test)]
pub mod tests {
    use std::{
        fs,
        sync::{Arc, Mutex},
    };

    use crate::db_manager::{
        creation::create_default_tables,
        helper::{gen_random_path, gen_vals},
        selecting::{calculate_links_from, find, get_new_link},
    };

    use super::is_finished;

    #[test]
    fn calculates_links() {
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
        prep.execute(("lp.ch", "help.ch")).unwrap();
        prep.execute(("ep.ch", "team-crystal.ch:::help.ch"))
            .unwrap();
        prep.execute(("l.ch", "help.ch:::google.ch")).unwrap();

        let test_result1 = calculate_links_from(&conn, 2).unwrap();

        let test_result2 = calculate_links_from(&conn, 1).unwrap();

        fs::remove_file(path).unwrap();

        assert_eq!(test_result1, 3);

        assert_eq!(test_result2, 1);
    }

    #[test]
    fn get_new_link_test() {
        let path = gen_random_path();
        let conn = create_default_tables(path.to_str().unwrap()).unwrap();
        let conn = Arc::new(Mutex::new(conn));
        {
            // fill db with values
            let prep = conn.lock().unwrap();
            let mut statement = prep
                .connection
                .prepare(
                    "INSERT INTO Ranking (url, links_to, in_use, visited) VALUES (?1, ?2, ?3, ?4)",
                )
                .unwrap();

            statement
                .execute((
                    "test.ch",
                    "team-crystal.ch:::google.ch:::example.com",
                    true,
                    false,
                ))
                .unwrap();
            statement
                .execute((
                    "help.ch",
                    "team-crystal.ch:::google.ch:::test.ch",
                    false,
                    false,
                ))
                .unwrap();
            statement
                .execute(("lp.ch", "help.ch", false, true))
                .unwrap();
            statement
                .execute(("ep.ch", "team-crystal.ch:::help.ch", false, false))
                .unwrap();
            statement
                .execute(("p.ch", "help.ch:::google.ch", true, true))
                .unwrap();
        }

        let link = get_new_link(conn).unwrap();
        assert_eq!(link.url, "help.ch");
        // had to drop here so that the database entry can be chanched before the file is deleted
        drop(link);
        fs::remove_file(path).unwrap();
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

        prep.execute((
            "test.ch",
            "team-crystal.ch:::google.ch:::example.com",
            true,
            false,
        ))
        .unwrap();
        prep.execute((
            "help.ch",
            "team-crystal.ch:::google.ch:::test.ch",
            false,
            false,
        ))
        .unwrap();
        prep.execute(("p.ch", "help.ch", false, true)).unwrap();
        prep.execute(("ep.ch", "team-crystal.ch:::help.ch", false, false))
            .unwrap();
        prep.execute(("lp.ch", "help.ch:::google.ch", true, true))
            .unwrap();
        fs::remove_file(path).unwrap();

        let result = is_finished(&conn).unwrap();

        assert!(!result)
    }

    #[test]
    fn is_finished_true() {
        let path = gen_random_path();

        let conn = create_default_tables(path.to_str().unwrap()).unwrap();

        // fill db with values
        let mut prep = conn
            .connection
            .prepare("INSERT INTO Ranking (url, links_to, in_use, visited) VALUES (?1, ?2, ?3, ?4)")
            .unwrap();

        prep.execute((
            "test.ch",
            "team-crystal.ch:::google.ch:::example.com",
            false,
            true,
        ))
        .unwrap();
        prep.execute((
            "help.ch",
            "team-crystal.ch:::google.ch:::test.ch",
            false,
            true,
        ))
        .unwrap();
        prep.execute(("p.ch", "help.ch", false, true)).unwrap();
        prep.execute(("ep.ch", "team-crystal.ch:::help.ch", false, true))
            .unwrap();
        prep.execute(("lp.ch", "help.ch:::google.ch", false, true))
            .unwrap();

        fs::remove_file(path).unwrap();
        let result = is_finished(&conn).unwrap();

        assert!(result)
    }

    #[test]
    fn search_function_test() {
        let path = gen_random_path();
        let conn = create_default_tables(path.to_str().unwrap()).unwrap();

        gen_vals(&conn);

        let test_results = find(&conn, "crystal").unwrap();

        assert_eq!(test_results.len(), 3);

        assert_eq!(test_results[0].id, 1);
        assert_eq!(test_results[1].id, 2);
        assert_eq!(test_results[2].id, 4);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn get_values_test() {
        todo!()

        // Test if get_values gets all values
    }
}
