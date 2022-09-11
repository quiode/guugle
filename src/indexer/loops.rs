use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    db_manager::{
        creation::{create_default_tables, unvisited_page, DatabaseConnection},
        selecting::{get_new_link, is_finished},
        updating::{set_in_use, update_to_visited},
    },
    indexer::visit_types::ToVisit,
    page_scraper::{
        html::{Html, HtmlGetterError},
        html_getter::html_getter,
        html_parser::get_links,
    },
};

pub fn run(start_urls: Vec<&str>, db_path: Option<&str>) {
    let db_path = db_path.unwrap_or("./database.db3");

    let conn = create_default_tables(db_path).unwrap();
    let conn = Arc::new(Mutex::new(conn));

    // fill in the start_urls
    for url in start_urls {
        unvisited_page(Arc::clone(&conn), url).unwrap();
    }

    cmd_fn(conn)
}

/// # Command function
///
/// 1. Stores all lists
/// 2. creates threads to parse new websites
fn cmd_fn(db_connection: Arc<Mutex<DatabaseConnection>>) {
    const THREAD_COUNT: i64 = 5;
    let mut threads = vec![];

    for _ in 0..THREAD_COUNT {
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

                let new_url = get_new_link(Arc::clone(&new_db_connection));
                let mut to_visit: ToVisit;

                match new_url {
                    Some(t_v) => to_visit = t_v,
                    None => {
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                }

                let mut html: Option<Html> = None;

                match rt.block_on(async { html_getter(&to_visit.url).await }) {
                    Ok(ok) => html = Some(ok),
                    Err(err) => match err {
                        HtmlGetterError::NotHTML => continue,
                        HtmlGetterError::GetError
                        | HtmlGetterError::StatusCode
                        | HtmlGetterError::UrlError => {
                            if to_visit.err_count > 3 {
                                // links doesn't work so just ignore it
                                update_to_visited(
                                    &new_db_connection.lock().unwrap(),
                                    to_visit.id,
                                    "",
                                    vec![],
                                )
                                .ok();
                                continue;
                            }

                            to_visit.err_count += 1;
                        }
                    },
                }

                let html = html.unwrap();

                let links = get_links(&html);

                update_to_visited(
                    &new_db_connection.lock().unwrap(),
                    to_visit.id,
                    &html.text,
                    links.iter().map(|string| string.as_str()).collect(),
                )
                .unwrap();

                // add newly found links to database
                for link in links {
                    unvisited_page(Arc::clone(&new_db_connection), &link).ok();
                }
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

    use crate::db_manager::{creation::create_default_tables, helper::*, selecting::get_values};

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

        fs::remove_file(path).unwrap();

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
