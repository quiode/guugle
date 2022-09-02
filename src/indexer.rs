use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::page_scraper::html_getter::HtmlGetterError;
use crate::page_scraper::html_parser::get_links_from_url;

pub fn run(start_urls: Vec<&str>) -> Vec<Visited> {
    let mut deque_urls: VecDeque<ToVisit> = VecDeque::new();

    for url in start_urls {
        deque_urls.push_back(url.to_string().into());
    }

    cmd_fn(deque_urls)
}

struct ToVisit {
    url: String,
    err_count: u8,
}

impl ToVisit {
    fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            err_count: 0,
        }
    }
}

impl From<String> for ToVisit {
    fn from(string: String) -> Self {
        Self::new(&string)
    }
}

#[derive(Debug)]
pub struct Visited {
    url: String,
}

impl Visited {
    fn new(visited: ToVisit) -> Self {
        Self { url: visited.url }
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
        }
    }
}

/// # Command function
///
/// 1. Stores all lists
/// 2. creates threads to parse new websites
fn cmd_fn(start_urls: VecDeque<ToVisit>) -> Vec<Visited> {
    // spawns list with sites to visit
    let to_visit: Arc<Mutex<VecDeque<ToVisit>>> = Arc::new(Mutex::new(start_urls));

    // spawns list with sites that have been visited
    let visited: Arc<Mutex<Vec<Visited>>> = Arc::new(Mutex::new(vec![]));

    let mut threads = vec![];

    for _ in 0..20 {
        let new_to_visit = Arc::clone(&to_visit);

        let new_visited = Arc::clone(&visited);

        threads.push(thread::spawn(move || {
            // After 5 attempts, exit loop
            let mut count = 0;

            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();

            loop {
                let new_url = new_to_visit.lock().unwrap().pop_front();
                let mut to_visit: ToVisit;

                match new_url {
                    Some(t_v) => to_visit = t_v,
                    None => {
                        if count > 5 {
                            break;
                        }
                        count += 1;
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                }

                count = 0;

                let mut links: Vec<String> = vec![];

                match rt.block_on(async { get_links_from_url(&to_visit.url).await }) {
                    Ok(ok) => links = ok,
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

                new_visited.lock().unwrap().push(to_visit.into());

                let mut to_visit = new_to_visit.lock().unwrap();

                for link in links {
                    to_visit.push_back(link.into());
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

    let mut result: Vec<Visited> = vec![];

    for val in visited.lock().unwrap().iter() {
        result.push(Visited::from(val))
    }

    result
}

#[cfg(test)]
mod tests {
    use super::run;

    #[test]
    fn run_fn_basic() {
        let start_urls = vec!["http://example.com/"];

        let result = run(start_urls);

        println!("{:?}", result);

        assert_eq!(result[0].url, "http://example.com/");
        assert_eq!(result[1].url, "https://www.iana.org/domains/example");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn run_fn_complex() {
        let start_urls = vec!["http://example.com/", "https://maslinks.netlify.app/"];

        let result = run(start_urls);

        let mut res = vec![];

        for visited in result.iter() {
            res.push(visited.url.clone());
        }

        eprintln!("{:?}", res);

        assert!(res.contains(&"http://example.com/".to_string()));
        assert!(res.contains(&"https://maslinks.netlify.app/".to_string()));
        assert!(res.contains(&"https://www.iana.org/domains/example".to_string()));
        assert!(res.contains(&"https://regameapp.vercel.app/".to_string()));
    }
}
