#[derive(Debug)]
pub struct Ranking {
    pub id: i64,
    pub visited: bool,
    pub url: String,
    pub content: Option<String>,
    pub links_to: Option<String>,
    pub in_use: bool,
}
