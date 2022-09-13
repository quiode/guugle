use std::sync::{Arc, Mutex};

use crate::db_manager::{creation::DatabaseConnection, updating::set_in_use};

pub struct ToVisit {
    pub url: String,
    pub id: i64,
    pub connection: Arc<Mutex<DatabaseConnection>>,
}

impl ToVisit {
    /// Panics if Mutex error
    pub fn new(
        url: &str,
        id: i64,
        connection: Arc<Mutex<DatabaseConnection>>,
    ) -> Result<Self, rusqlite::Error> {
        set_in_use(&connection.lock().unwrap(), id, true)?;

        Ok(Self {
            url: url.to_string(),
            id,
            connection,
        })
    }
}

impl Drop for ToVisit {
    /// Panics if database execution didn't work
    fn drop(&mut self) {
        set_in_use(&self.connection.lock().unwrap(), self.id, false).unwrap();
    }
}

#[derive(Debug)]
pub struct Visited {
    url: String,
    id: i64,
}

impl Visited {
    pub fn new(visited: ToVisit) -> Self {
        Self {
            url: visited.url.clone(),
            id: visited.id,
        }
    }
}

impl From<ToVisit> for Visited {
    fn from(visited: ToVisit) -> Self {
        Self::new(visited)
    }
}

impl From<&Visited> for Visited {
    fn from(visited: &Visited) -> Self {
        Self {
            url: visited.url.clone(),
            id: visited.id,
        }
    }
}
