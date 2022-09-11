use super::html_parser::is_html;

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
        if !is_html(text) {
            panic!("Not valid HTML!");
        }

        Html {
            text: text.to_string(),
            _private: (),
        }
    }
}

#[derive(Debug)]
pub enum HtmlGetterError {
    NotHTML,
    GetError,
    StatusCode,
    UrlError,
}
