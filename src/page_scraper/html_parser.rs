/// Parses a string containing html and returns all links
pub fn getLinks(html: Html) -> Vec<String> {
    let splits = html.text.split("<a href=\"");

    let mut links: Vec<String> = vec![];

    for split in splits {
        links.push(split.split("\"").next().unwrap().to_string())
    }

    links
}

/// Parses a string and checks if it is valid html (correct doctype)
fn isHtml(html: &str) -> bool {
    let start_string = "<!doctype html>";

    html.to_ascii_lowercase()
        .trim_start()
        .starts_with(start_string)
}

pub struct Html {
    pub text: String,
    _private: (),
}

/// Creates new Html instance from a string
///
/// # Panics
/// Panics if the string isn't valid html
impl Html {
    pub fn new(text: &str) -> Self {
        if !isHtml(text) {
            panic!("Not valid HTML!");
        }

        Html {
            text: text.to_string(),
            _private: (),
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn valid_html() {
        let html = r#"
        <!doctype html><html itemscope="" itemtype="http://schema.org/WebPage" lang="de-CH"><head><meta content="text/html; charset=UTF-8" http-equiv="Content-Type"><meta content="/images/branding/googleg/1x/googleg_standard_color_128dp.png" itemprop="image"><title>Google</title>
        "#;

        assert!(super::isHtml(html));
    }

    #[test]
    fn invalid_html() {
        let html = r#"
        <html itemscope="" itemtype="http://schema.org/WebPage" lang="de-CH"><head><meta content="text/html; charset=UTF-8" http-equiv="Content-Type"><meta content="/images/branding/googleg/1x/googleg_standard_color_128dp.png" itemprop="image"><title>Google</title>
        "#;

        assert!(!super::isHtml(html))
    }

    #[test]
    fn get_links() {
        let html = r#"
        <!DOCTYPE html>
        <html>
        <body>
        <a href="www.google.ch"></a>
        <h1>
            <a href="https://team-crystal.ch"></a>
        </h1>
        </body>
        </html>
        "#;

        let links = super::getLinks(super::Html::new(html));

        assert!(links.contains(&"www.google.ch".to_string()));
        assert!(links.contains(&"https://team-crystal.ch".to_string()));
    }
}
