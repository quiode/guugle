use super::{
    html::{Html, HtmlGetterError},
    html_getter::html_getter,
};

/// Parses a string containing html and returns all links
pub fn get_links(html: &Html) -> Vec<String> {
    let mut start_pos;
    let mut end_pos;

    let mut string = html.text.clone();

    let mut links = vec![];

    loop {
        // go to the next link tag and remove previous
        if let Some(i) = string.find("<a") {
            start_pos = i;
        } else {
            break;
        }

        string = string[start_pos + "<a".len()..].to_string();

        // go to ref section of link tag and remove previous
        if let Some(i) = string.find("href=\"") {
            start_pos = i;
        } else {
            break;
        }

        string = string[start_pos + "href=\"".len()..].to_string();

        // find ending tag of url
        if let Some(i) = string.find("\"") {
            end_pos = i;
        } else {
            break;
        }

        links.push(string[..end_pos].to_string());
    }

    links
}

pub async fn get_links_from_url(url: &str) -> Result<Vec<String>, HtmlGetterError> {
    let html = html_getter(url).await?;

    let links = get_links(&html);

    Ok(links)
}

/// Parses a string and checks if it is valid html (correct doctype)
pub fn is_html(html: &str) -> bool {
    let start_string = "<!doctype html>";

    html.to_ascii_lowercase()
        .trim_start()
        .starts_with(start_string)
}

#[cfg(test)]
mod tests {

    #[test]
    fn valid_html() {
        let html = r#"
        <!doctype html><html itemscope="" itemtype="http://schema.org/WebPage" lang="de-CH"><head><meta content="text/html; charset=UTF-8" http-equiv="Content-Type"><meta content="/images/branding/googleg/1x/googleg_standard_color_128dp.png" itemprop="image"><title>Google</title>
        "#;

        assert!(super::is_html(html));
    }

    #[test]
    fn invalid_html() {
        let html = r#"
        <html itemscope="" itemtype="http://schema.org/WebPage" lang="de-CH"><head><meta content="text/html; charset=UTF-8" http-equiv="Content-Type"><meta content="/images/branding/googleg/1x/googleg_standard_color_128dp.png" itemprop="image"><title>Google</title>
        "#;

        assert!(!super::is_html(html))
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

        let links = super::get_links(&super::Html::new(html));

        assert!(links.contains(&"www.google.ch".to_string()));
        assert!(links.contains(&"https://team-crystal.ch".to_string()));
    }

    #[tokio::test]
    async fn get_links_from_url() {
        let url = "example.com";

        let links = super::get_links_from_url(url).await.unwrap();

        assert_eq!(links[0], "https://www.iana.org/domains/example".to_string());
        assert_eq!(links.len(), 1);
    }

    #[tokio::test]
    async fn get_multiple_links_from_url() {
        let url = "https://maslinks.netlify.app/";

        let links = super::get_links_from_url(url).await.unwrap();

        assert_eq!(
            links,
            Vec::from(["https://franciscofunes.netlify.app/", "https://www.equaldev.com/", "https://creatumeme.netlify.app/", "https://flippingcard.netlify.app/", "https://dejalo-ir.herokuapp.com/", "https://regameapp.vercel.app/", "https://www.canva.com/design/DAEaKUy8pIc/_Ojr-mEqVtj3v1hiM_cPfg/view?utm_content=DAEaKUy8pIc&amp;utm_campaign=designshare&amp;utm_medium=link&amp;utm_source=publishpresent", "https://www.canva.com/design/DAEmkswvTqY/CTTmlrWqqg34YeZzoekonQ/view?utm_content=DAEmkswvTqY&utm_campaign=designshare&utm_medium=link&utm_source=publishsharelink", "https://www.canva.com/design/DAE2AP2m6aA/fffDzntOgOx7pLjGMH7msQ/view#7", "https://www.canva.com/design/DAE3VxjZRNE/Jcp8FgPK33SwrCNtR94KGw/view?utm_content=DAE3VxjZRNE&utm_campaign=designshare&utm_medium=link&utm_source=viewer", "https://www.canva.com/design/DAErbz8ncho/ngoanfXpz_xuerZ24ZqUhQ/view?utm_content=DAErbz8ncho&utm_campaign=designshare&utm_medium=link&utm_source=publishsharelink", "mailto:f.funes@bue.edu.ar", "https://wa.link/6kbz3s", "https://es.linkedin.com/in/francisco-funes/", "https://instagram.com/francisco_ign_/", "https://github.com/francisco-funes/", "https://codepen.io/franfunes"])
        );
    }
}
