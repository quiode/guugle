use std::{
    collections::LinkedList,
    sync::{mpsc, Arc, Mutex},
    thread,
};

use guugle::get_links;

struct ChannelMessage {
    link: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. make mutex with all links that have to be visited
    // 2. make list with all visited links
    // 3. tx pops one new link from mutex, visits it and sends info to receiver, gets new link, repeat
    // 4. rx gets all visited links, adds them to another list (in the future with other data)
    // 5. rx adds all newly discovered links to the mutex

    let to_visit = Arc::new(Mutex::new(LinkedList::<String>::new()));
    let visited = Arc::new(Mutex::new(LinkedList::<String>::new()));

    let (tx, rx) = mpsc::channel::<ChannelMessage>();

    let mut handles = vec![];

    for _ in 0..20 {
        let tx_c = tx.clone();
        let to_visit_c = Arc::clone(&to_visit);
        let visited_c = Arc::clone(&visited);

        handles.push(thread::spawn(|| async move {
            let link = to_visit_c.lock().unwrap().pop_front();

            if let Some(link) = link {
                let links = get_links(&link).await;
                visited_c.lock().unwrap().push_back(link);

                for link in links {
                    tx_c.send(ChannelMessage { link }).ok();
                }
            }
        }));
    }

    for message in rx {
        to_visit.lock().unwrap().push_back(message.link);
        println!("{:?}", visited);
    }

    for handle in handles {
        handle.join();
    }

    Ok(())
}
