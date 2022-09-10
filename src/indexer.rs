use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::db_manager::{get_new_link, DatabaseConnection};
use crate::{
    db_manager::{create_default_tables, is_finished, unvisited_page, update_to_visited},
    page_scraper::{
        html_getter::{html_getter, HtmlGetterError},
        html_parser::{get_links, Html},
    },
};

pub fn run(start_urls: Vec<&str>, db_path: Option<&str>) {
    let db_path = db_path.unwrap_or("./database.db3");

    let conn = create_default_tables(db_path).unwrap();

    // fill in the start_urls
    for url in start_urls {
        unvisited_page(&conn, url).unwrap();
    }

    cmd_fn(conn);
}

pub struct ToVisit {
    pub url: String,
    pub id: i64,
    err_count: u8,
}

impl ToVisit {
    pub fn new(url: &str, id: i64) -> Self {
        Self {
            url: url.to_string(),
            err_count: 0,
            id,
        }
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
            url: visited.url,
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

/// # Command function
///
/// 1. Stores all lists
/// 2. creates threads to parse new websites
fn cmd_fn(db_connection: DatabaseConnection) {
    let db_connection = Arc::new(Mutex::new(db_connection));
    let mut threads = vec![];

    for _ in 0..20 {
        let new_db_connection = Arc::clone(&db_connection);

        threads.push(thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();

            loop {
                if is_finished(&new_db_connection.lock().unwrap()).unwrap_or(false) {
                    break;
                }

                let new_url = get_new_link(&new_db_connection.lock().unwrap());
                let mut to_visit: ToVisit;

                match new_url {
                    Some(t_v) => to_visit = t_v,
                    None => {
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                }

                let mut html = Html::new("");

                match rt.block_on(async { html_getter(&to_visit.url).await }) {
                    Ok(ok) => html = ok,
                    Err(err) => match err {
                        HtmlGetterError::NotHTML => continue,
                        HtmlGetterError::GetError
                        | HtmlGetterError::StatusCode
                        | HtmlGetterError::UrlError => {
                            if to_visit.err_count > 3 {
                                // links doesn't work so just ignore it
                                continue;
                            }

                            to_visit.err_count += 1;
                        }
                    },
                }

                let links = get_links(&html);

                update_to_visited(
                    &new_db_connection.lock().unwrap(),
                    to_visit.id,
                    &html.text,
                    links.iter().map(|string| string.as_str()).collect(),
                )
                .unwrap();
            }
        }));
    }

    for thread in threads {
        match thread.join() {
            Ok(_) => {}
            Err(err) => eprintln!("{:#?}", err),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::db_manager::{create_default_tables, get_values, tests::gen_random_path};

    // TODO: Rewrite tests so that they work with the database
    // TODO: Add tests to test for content/corret status
    use super::run;

    #[test]
    fn run_fn_basic() {
        let start_urls = vec!["http://example.com/"];

        let path = gen_random_path();

        run(start_urls, Some(path.to_str().unwrap()));

        let conn = create_default_tables(path.to_str().unwrap()).unwrap();

        let result = get_values(&conn).unwrap();

        println!("{:?}", result);

        // fs::remove_file(path).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].url, "http://example.com/");
        assert_eq!(result[1].url, "https://www.iana.org/domains/example");
    }

    #[test]
    fn run_fn_complex() {
        let start_urls = vec!["http://example.com/", "https://maslinks.netlify.app/"];

        let path = gen_random_path();

        let conn = create_default_tables(path.to_str().unwrap()).unwrap();

        run(start_urls, Some(path.to_str().unwrap()));

        let res = get_values(&conn).unwrap();

        let mut res = res.iter();

        eprintln!("{:?}", res);

        fs::remove_file(path).unwrap();

        assert!(res.any(|res| res.url == "http://example.com/".to_string()));
        assert!(res.any(|res| res.url == "https://maslinks.netlify.app/".to_string()));
        assert!(res.any(|res| res.url == "https://www.iana.org/domains/example".to_string()));
        assert!(res.any(|res| res.url == "https://regameapp.vercel.app/".to_string()));
    }
}
