use hyper::Client;
use hyper_tls::HttpsConnector;

use super::html_parser::Html;

#[derive(Debug)]
enum HtmlGetterError {
    NotHTML,
    GetError,
}

/// # Returns valid html from a link or an error if the page isn't html
///
/// # Panics
/// Panics if the uri can't be parsed or the bytes can't be read
async fn html_getter(link: &str) -> Result<Html, HtmlGetterError> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let uri = link.parse().unwrap();

    let response = client.get(uri).await;

    if let Ok(ok) = response {
        let mut response = ok;

        if response.headers()["content-type"] != "text/html; charset=UTF-8" {
            return Err(HtmlGetterError::NotHTML);
        }

        let bytes = hyper::body::to_bytes(response.body_mut()).await.unwrap();

        let text = String::from_utf8(bytes.to_vec()).unwrap();

        return Ok(Html::new(&text));
    } else {
        return Err(HtmlGetterError::GetError);
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn html_getter_http() {
        let uri = "http://example.com/";

        let body = "<!doctype html>\n<html>\n<head>\n    <title>Example Domain</title>\n\n    <meta charset=\"utf-8\" />\n    <meta http-equiv=\"Content-type\" content=\"text/html; charset=utf-8\" />\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n    <style type=\"text/css\">\n    body {\n        background-color: #f0f0f2;\n        margin: 0;\n        padding: 0;\n        font-family: -apple-system, system-ui, BlinkMacSystemFont, \"Segoe UI\", \"Open Sans\", \"Helvetica Neue\", Helvetica, Arial, sans-serif;\n        \n    }\n    div {\n        width: 600px;\n        margin: 5em auto;\n        padding: 2em;\n        background-color: #fdfdff;\n        border-radius: 0.5em;\n        box-shadow: 2px 3px 7px 2px rgba(0,0,0,0.02);\n    }\n    a:link, a:visited {\n        color: #38488f;\n        text-decoration: none;\n    }\n    @media (max-width: 700px) {\n        div {\n            margin: 0 auto;\n            width: auto;\n        }\n    }\n    </style>    \n</head>\n\n<body>\n<div>\n    <h1>Example Domain</h1>\n    <p>This domain is for use in illustrative examples in documents. You may use this\n    domain in literature without prior coordination or asking for permission.</p>\n    <p><a href=\"https://www.iana.org/domains/example\">More information...</a></p>\n</div>\n</body>\n</html>\n";

        let result = super::html_getter(uri).await.unwrap();

        assert_eq!(result.text, body);
    }

    #[tokio::test]
    async fn html_getter_https() {
        let uri = "https://example.com/";

        let body = "<!doctype html>\n<html>\n<head>\n    <title>Example Domain</title>\n\n    <meta charset=\"utf-8\" />\n    <meta http-equiv=\"Content-type\" content=\"text/html; charset=utf-8\" />\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n    <style type=\"text/css\">\n    body {\n        background-color: #f0f0f2;\n        margin: 0;\n        padding: 0;\n        font-family: -apple-system, system-ui, BlinkMacSystemFont, \"Segoe UI\", \"Open Sans\", \"Helvetica Neue\", Helvetica, Arial, sans-serif;\n        \n    }\n    div {\n        width: 600px;\n        margin: 5em auto;\n        padding: 2em;\n        background-color: #fdfdff;\n        border-radius: 0.5em;\n        box-shadow: 2px 3px 7px 2px rgba(0,0,0,0.02);\n    }\n    a:link, a:visited {\n        color: #38488f;\n        text-decoration: none;\n    }\n    @media (max-width: 700px) {\n        div {\n            margin: 0 auto;\n            width: auto;\n        }\n    }\n    </style>    \n</head>\n\n<body>\n<div>\n    <h1>Example Domain</h1>\n    <p>This domain is for use in illustrative examples in documents. You may use this\n    domain in literature without prior coordination or asking for permission.</p>\n    <p><a href=\"https://www.iana.org/domains/example\">More information...</a></p>\n</div>\n</body>\n</html>\n";

        let result = super::html_getter(uri).await.unwrap();

        assert_eq!(result.text, body);
    }
}
