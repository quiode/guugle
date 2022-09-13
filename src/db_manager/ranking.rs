#[derive(Debug)]
pub struct Ranking {
    pub id: i64,
    pub visited: bool,
    pub url: String,
    pub content: Option<String>,
    pub links_to: Option<String>,
    pub in_use: bool,
}

impl PartialEq for Ranking {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.visited == other.visited
            && self.url == other.url
            && self.content == other.content
            && self.links_to == other.links_to
            && self.in_use == other.in_use
    }
}
