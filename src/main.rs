use std::{collections::LinkedList, sync::Mutex, thread};

use guugle::get_links;

struct Link {
    visited: bool,
    link: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // first is visited links, last is links that need visiting
    let mut links: Mutex<(Vec<String>, LinkedList<String>)> = Mutex::new((
        vec![],
        LinkedList::from_iter(
            [
                "https://dominik-schwaiger.ch".to_string(),
                "https://en.wikipedia.org/wiki/Social_media".to_string(),
            ]
            .into_iter(),
        ),
    ));

    for _ in 0..20 {
        thread::spawn(|| async {
            let mut m = links.lock().unwrap();
            let link = m.1.pop_front();

            if let Some(link) = link {
                let new_links = get_links(&link).await;

                for l in new_links {
                    m.1.push_back(l);
                }
            }
        });
    }

    Ok(())
}
