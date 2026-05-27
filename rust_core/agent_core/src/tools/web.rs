use serde::{Deserialize, Serialize};
use std::os::raw::c_char;

#[derive(Debug, Serialize, Deserialize)]
pub struct WebSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebFetchResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Web tools that delegate HTTP to the host via the FFI post_fn callback.
///
/// Instead of bundling an HTTP client (which adds complexity for cross-platform),
/// the web tools call back into the host's native HTTP stack (ArkTS `@ohos.net.http`,
/// Node.js `fetch`, or WASM `XMLHttpRequest`).
pub struct WebTools {
    /// Optional callback for making HTTP requests through the host.
    /// Signature: `fn(url: &str, body: &str) -> String` (returns response JSON).
    post_fn: Option<extern "C" fn(url: *const c_char, body: *const c_char) -> *mut c_char>,
}

impl WebTools {
    pub fn new(
        post_fn: Option<extern "C" fn(url: *const c_char, body: *const c_char) -> *mut c_char>,
    ) -> Self {
        Self { post_fn }
    }

    pub fn web_search(&self, query: &str) -> Result<Vec<WebSearchResult>, String> {
        let Some(post_fn) = self.post_fn else {
            return Err("Web search requires host IO callback (post_fn)".into());
        };

        let url = "https://html.duckduckgo.com/html/";
        let body = format!("q={}", urlencoding(query));
        let url_c = std::ffi::CString::new(url).unwrap();
        let body_c = std::ffi::CString::new(body).unwrap();

        let resp_ptr = post_fn(url_c.as_ptr(), body_c.as_ptr());
        if resp_ptr.is_null() {
            return Err("Host IO returned null".into());
        }

        let resp = unsafe { std::ffi::CStr::from_ptr(resp_ptr) }
            .to_str()
            .unwrap_or("")
            .to_string();

        // Parse DuckDuckGo HTML results
        Self::parse_ddg_html(&resp)
    }

    pub fn web_fetch(&self, url: &str) -> WebFetchResult {
        let Some(post_fn) = self.post_fn else {
            return WebFetchResult {
                success: false,
                content: None,
                error: Some("Host IO callback not available".into()),
            };
        };

        let url_c = std::ffi::CString::new(url).unwrap();
        let body_c = std::ffi::CString::new("").unwrap();
        let resp_ptr = post_fn(url_c.as_ptr(), body_c.as_ptr());

        if resp_ptr.is_null() {
            return WebFetchResult {
                success: false,
                content: None,
                error: Some("Host IO returned null".into()),
            };
        }

        let resp = unsafe { std::ffi::CStr::from_ptr(resp_ptr) }
            .to_str()
            .unwrap_or("")
            .to_string();

        // Attempt to extract text from HTML
        let text = Self::strip_html(&resp);
        WebFetchResult {
            success: true,
            content: Some(if text.len() > 8000 { text[..8000].into() } else { text }),
            error: None,
        }
    }

    fn parse_ddg_html(html: &str) -> Result<Vec<WebSearchResult>, String> {
        let mut results: Vec<WebSearchResult> = vec![];

        // Simple regex-free extraction of DDG result snippets
        // Each result is in a div with class "result"
        for chunk in html.split("class=\"result__body\"") {
            if results.len() >= 10 {
                break;
            }

            let title = extract_between(chunk, "class=\"result__a\"", "</a>")
                .and_then(|a| extract_between(&a, ">", "<"))
                .unwrap_or_default();

            let url = extract_between(chunk, "class=\"result__url\"", "</")
                .and_then(|u| extract_between(&u, ">", "<"))
                .unwrap_or_default();

            let snippet = extract_between(chunk, "class=\"result__snippet\"", "</")
                .and_then(|s| extract_between(&s, ">", "<"))
                .unwrap_or_default();

            if !title.is_empty() {
                results.push(WebSearchResult {
                    title: htmldecode(&title),
                    url: url.trim().to_string(),
                    snippet: htmldecode(&snippet),
                });
            }
        }

        Ok(results)
    }

    fn strip_html(html: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;
        for ch in html.chars() {
            if ch == '<' {
                in_tag = true;
            } else if ch == '>' {
                in_tag = false;
                result.push(' ');
            } else if !in_tag {
                result.push(ch);
            }
        }
        // Collapse whitespace
        let words: Vec<&str> = result.split_whitespace().collect();
        words.join(" ")
    }
}

fn extract_between<'a>(haystack: &'a str, start_pat: &str, end_pat: &str) -> Option<String> {
    let start = haystack.find(start_pat)? + start_pat.len();
    let rest = &haystack[start..];
    let end = rest.find(end_pat)?;
    Some(rest[..end].to_string())
}

fn urlencoding(s: &str) -> String {
    let mut result = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(b as char);
            }
            b' ' => result.push('+'),
            _ => result.push_str(&format!("%{:02X}", b)),
        }
    }
    result
}

fn htmldecode(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
}
