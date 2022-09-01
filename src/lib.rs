mod page_rank;
mod page_scraper;

pub async fn get_links(url: &str) -> Vec<String> {
    let html = page_scraper::html_getter::html_getter(url).await.unwrap();

    let links = page_scraper::html_parser::getLinks(html);

    links
}
