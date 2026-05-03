use url::Url;

pub(crate) fn normalize_server_url(url: &str) -> Result<String, String> {
    let trimmed = url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err("Server URL is required.".into());
    }

    let parsed =
        Url::parse(trimmed).map_err(|_| "Enter a valid http:// or https:// URL.".to_string())?;
    match parsed.scheme() {
        "http" | "https" => {}
        _ => return Err("Server URL must start with http:// or https://.".into()),
    }
    if parsed.host_str().is_none() {
        return Err("Server URL must include a hostname.".into());
    }

    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::normalize_server_url;

    #[test]
    fn trims_whitespace_and_trailing_slashes() {
        assert_eq!(
            normalize_server_url("  https://chat.example.com///  "),
            Ok("https://chat.example.com".to_string())
        );
    }

    #[test]
    fn accepts_http_and_https_urls_with_hosts() {
        assert_eq!(
            normalize_server_url("http://localhost:3000"),
            Ok("http://localhost:3000".to_string())
        );
        assert_eq!(
            normalize_server_url("https://chat.example.com/workspace"),
            Ok("https://chat.example.com/workspace".to_string())
        );
    }

    #[test]
    fn rejects_blank_urls() {
        assert_eq!(
            normalize_server_url("   "),
            Err("Server URL is required.".to_string())
        );
    }

    #[test]
    fn rejects_invalid_urls() {
        assert_eq!(
            normalize_server_url("not a url"),
            Err("Enter a valid http:// or https:// URL.".to_string())
        );
    }

    #[test]
    fn rejects_non_http_schemes() {
        assert_eq!(
            normalize_server_url("ftp://chat.example.com"),
            Err("Server URL must start with http:// or https://.".to_string())
        );
    }

    #[test]
    fn rejects_urls_without_hosts() {
        assert_eq!(
            normalize_server_url("http://"),
            Err("Enter a valid http:// or https:// URL.".to_string())
        );
    }
}
