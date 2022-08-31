use hyper::Client;

#[derive(Debug)]
enum HtmlGetterError {
    NotHTML,
    GetError,
}

/// # Returns valid html from a link or an error if the page isn't html
///
/// # Panics
/// Panics if the uri can't be parsed or the bytes can't be read
async fn html_getter(link: &str) -> Result<String, HtmlGetterError> {
    let client = Client::new();

    let uri = link.parse().unwrap();

    let response = client.get(uri).await;

    if let Ok(ok) = response {
        let mut response = ok;

        if response.headers()["content-type"] != "text/html; charset=UTF-8" {
            return Err(HtmlGetterError::NotHTML);
        }

        let bytes = hyper::body::to_bytes(response.body_mut()).await.unwrap();

        let text = String::from_utf8(bytes.to_vec()).unwrap();

        return Ok(text);
    } else {
        return Err(HtmlGetterError::GetError);
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn html_getter_google() {
        let uri = "http://google.ch";

        let body = "<HTML><HEAD><meta http-equiv=\"content-type\" content=\"text/html;charset=utf-8\">\n<TITLE>301 Moved</TITLE></HEAD><BODY>\n<H1>301 Moved</H1>\nThe document has moved\n<A HREF=\"http://www.google.ch/\">here</A>.\r\n</BODY></HTML>\r\n";

        let result = super::html_getter(uri).await.unwrap();

        assert_eq!(result, body);
    }
}
