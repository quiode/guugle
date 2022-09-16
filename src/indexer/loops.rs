use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    db_manager::{
        creation::{create_default_tables, unvisited_page, DatabaseConnection},
        selecting::{get_new_link, is_finished},
        updating::update_to_visited,
    },
    indexer::visit_types::ToVisit,
    page_scraper::{
        html::{Html, HtmlGetterError},
        html_getter::html_getter,
        html_parser::get_links,
    },
};

pub fn run(start_urls: Vec<&str>, db_path: Option<String>, verbose: bool) {
    let db_path = db_path.unwrap_or("./database.db3".to_owned());

    let conn = create_default_tables(&db_path).unwrap();
    let conn = Arc::new(Mutex::new(conn));

    // fill in the start_urls
    for url in start_urls {
        unvisited_page(Arc::clone(&conn), url, verbose).unwrap();
    }

    cmd_fn(conn, verbose)
}

/// # Command function
///
/// 1. Stores all lists
/// 2. creates threads to parse new websites
fn cmd_fn(db_connection: Arc<Mutex<DatabaseConnection>>, verbose: bool) {
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
                    if verbose {
                        println!("No new pages to crawl found, shutting down thread...");
                    }
                    break;
                }

                let new_url = get_new_link(Arc::clone(&new_db_connection));
                let to_visit: ToVisit;

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
                        HtmlGetterError::NotHTML => {
                            update_to_visited(
                                &new_db_connection.lock().unwrap(),
                                to_visit.id,
                                "NOT HTML",
                                vec![],
                                verbose,
                            )
                            .ok();
                            continue;
                        }
                        HtmlGetterError::GetError
                        | HtmlGetterError::StatusCode
                        | HtmlGetterError::UrlError => {
                            // links doesn't work so just ignore it
                            update_to_visited(
                                &new_db_connection.lock().unwrap(),
                                to_visit.id,
                                "ERROR",
                                vec![],
                                verbose,
                            )
                            .ok();

                            continue;
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
                    verbose,
                )
                .unwrap();

                // add newly found links to database
                for link in links {
                    unvisited_page(Arc::clone(&new_db_connection), &link, verbose).ok();
                }
            }
        }));
    }

    for thread in threads {
        match thread.join() {
            Ok(_) => {}
            Err(err) => println!("{:#?}", err),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::hash_map::DefaultHasher, fs, hash::Hash};

    use hex::{encode, ToHex};
    use sha2::{Digest, Sha256};

    use super::run;
    use crate::db_manager::{creation::create_default_tables, helper::*, selecting::get_values};

    #[test]
    fn run_fn_basic_urls() {
        let start_urls = vec!["http://example.com/"];

        let path = gen_random_path();

        run(start_urls, Some(path.to_str().unwrap().to_string()), false);

        let conn = create_default_tables(path.to_str().unwrap()).unwrap();

        let result = get_values(&conn).unwrap();

        println!("{:?}", result);

        fs::remove_file(path).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].url, "http://example.com/");
        assert_eq!(result[1].url, "https://www.iana.org/domains/example");
    }

    /// checks if content is correct
    #[test]
    fn run_fn_basic_content() {
        let start_urls = vec!["http://example.com/"];

        let path = gen_random_path();

        run(start_urls, Some(path.to_str().unwrap().to_string()), false);

        let conn = create_default_tables(path.to_str().unwrap()).unwrap();

        let result = get_values(&conn).unwrap();

        println!("{:?}", result);

        let content = [
            r#"
        <!doctype html>
<html>
<head>
    <title>Example Domain</title>

    <meta charset="utf-8" />
    <meta http-equiv="Content-type" content="text/html; charset=utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <style type="text/css">
    body {
        background-color: #f0f0f2;
        margin: 0;
        padding: 0;
        font-family: -apple-system, system-ui, BlinkMacSystemFont, "Segoe UI", "Open Sans", "Helvetica Neue", Helvetica, Arial, sans-serif;
        
    }
    div {
        width: 600px;
        margin: 5em auto;
        padding: 2em;
        background-color: #fdfdff;
        border-radius: 0.5em;
        box-shadow: 2px 3px 7px 2px rgba(0,0,0,0.02);
    }
    a:link, a:visited {
        color: #38488f;
        text-decoration: none;
    }
    @media (max-width: 700px) {
        div {
            margin: 0 auto;
            width: auto;
        }
    }
    </style>    
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
        "#.trim(),
            "ERROR",
        ];

        fs::remove_file(path).unwrap();

        assert_eq!(result[0].content.as_ref().unwrap().trim(), content[0]);
        assert_eq!(result[1].content.as_ref().unwrap().trim(), content[1]);
    }

    #[test]
    fn run_fn_complex_urls() {
        let start_urls = vec!["http://example.com/", "https://maslinks.netlify.app/"];

        let path = gen_random_path();

        let conn = create_default_tables(path.to_str().unwrap()).unwrap();

        run(start_urls, Some(path.to_str().unwrap().to_string()), false);

        let res = get_values(&conn).unwrap();

        let mut res = res.iter();

        println!("{:?}", res);

        fs::remove_file(path).unwrap();

        assert!(res.any(|res| res.url == "http://example.com/".to_string()));
        assert!(res.any(|res| res.url == "https://maslinks.netlify.app/".to_string()));
        assert!(res.any(|res| res.url == "https://www.iana.org/domains/example".to_string()));
        assert!(res.any(|res| res.url == "https://regameapp.vercel.app/".to_string()));
    }

    /// checks if content is correct
    #[test]
    fn run_fn_complex_content() {
        let hasher = Sha256::new();

        let start_urls = vec!["http://example.com/", "https://maslinks.netlify.app/"];

        let path = gen_random_path();

        let conn = create_default_tables(path.to_str().unwrap()).unwrap();

        run(start_urls, Some(path.to_str().unwrap().to_string()), false);

        let res = get_values(&conn).unwrap();

        let mut res = res.iter();

        let correct_content = ["Página web gratuita para crear Memes", "<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css?family=Mukta:300,400,700\">"];

        println!("{:#?}", res);

        assert!(res.any(|res| {
            if res.url != "https://creatumeme.netlify.app/" {
                return false;
            }

            match &res.content {
                None => return false,
                Some(c) => return c.contains(correct_content[0]),
            }
        }));

        assert!(res.any(|res| {
            if res.url != "https://dejalo-ir.herokuapp.com/" {
                return false;
            }

            match &res.content {
                None => return false,
                Some(c) => return c.contains(correct_content[1]),
            }
        }));

        fs::remove_file(path).unwrap();
    }
}
