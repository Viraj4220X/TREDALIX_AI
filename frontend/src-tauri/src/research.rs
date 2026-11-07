use reqwest::{Client, Response, StatusCode, Url, redirect::Policy};
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, SocketAddr, ToSocketAddrs},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const GOOGLE_SEARCH_URL: &str = "https://www.googleapis.com/customsearch/v1";
const MAX_QUERIES: usize = 8;
const MAX_QUERY_BYTES: usize = 240;
const MAX_SOURCES: usize = 8;
const MAX_RESPONSE_BYTES: usize = 512 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(12);
const MAX_CSE_ITEMS: u8 = 5;

#[derive(Debug, Clone, Serialize)]
pub struct ResearchStatus {
    pub configured: bool,
    pub provider: &'static str,
    pub notice: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResearchSource {
    pub id: String,
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub excerpt_html: Option<String>,
    pub retrieved_at_utc: i64,
    pub retrieval_status: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResearchResult {
    pub provider: &'static str,
    pub queries: Vec<String>,
    pub sources: Vec<ResearchSource>,
    pub pages_read: usize,
    pub pages_unavailable: usize,
    pub retrieved_at_utc: i64,
}

#[derive(Deserialize)]
struct GoogleResponse {
    items: Option<Vec<GoogleItem>>,
}

#[derive(Deserialize)]
struct GoogleItem {
    title: Option<String>,
    link: Option<String>,
    snippet: Option<String>,
}

struct SearchItem {
    title: String,
    url: String,
    snippet: String,
}

trait SearchProvider: Send + Sync {
    fn id(&self) -> &'static str;

    async fn search(&self, queries: &[String]) -> Result<Vec<SearchItem>, String>;
}

struct GoogleCseProvider {
    key: String,
    engine: String,
}

impl GoogleCseProvider {
    fn from_environment() -> Option<Self> {
        let (key, engine) = credentials()?;
        Some(Self { key, engine })
    }
}

impl SearchProvider for GoogleCseProvider {
    fn id(&self) -> &'static str {
        "google_cse"
    }

    async fn search(&self, queries: &[String]) -> Result<Vec<SearchItem>, String> {
        let client = make_client()?;
        let mut items = Vec::new();
        for query in queries {
            for item in fetch_search_results(&client, &self.key, &self.engine, query).await? {
                let (Some(title), Some(url)) = (item.title, item.link) else {
                    continue;
                };
                items.push(SearchItem {
                    title,
                    url,
                    snippet: item.snippet.unwrap_or_default(),
                });
                if items.len() == MAX_SOURCES {
                    return Ok(items);
                }
            }
        }
        Ok(items)
    }
}

pub fn status() -> ResearchStatus {
    status_for(GoogleCseProvider::from_environment().is_some())
}

fn status_for(configured: bool) -> ResearchStatus {
    ResearchStatus {
        configured,
        provider: if configured {
            "google_cse"
        } else {
            "not_configured"
        },
        notice: if configured {
            "Local Google CSE settings are present, but credentials have not been tested. Google Programmable Search is a legacy adapter scheduled to discontinue on 2027-01-01."
        } else {
            "Web Search Not Configured. No search requests will be made. Configure a supported provider locally to enable web search; local Gemma chat is independent."
        },
    }
}

fn credentials() -> Option<(String, String)> {
    let key = std::env::var("TREDALIX_GOOGLE_CSE_API_KEY").ok()?;
    let engine = std::env::var("TREDALIX_GOOGLE_CSE_CX").ok()?;
    if key.trim().is_empty() || key.len() > 256 || engine.trim().is_empty() || engine.len() > 256 {
        return None;
    }
    Some((key, engine))
}

fn validate_queries(queries: &[String]) -> Result<Vec<String>, String> {
    if queries.is_empty() || queries.len() > MAX_QUERIES {
        return Err("Enter 1–8 focused search queries.".into());
    }
    let mut unique = Vec::with_capacity(queries.len());
    for query in queries {
        let query = query.trim();
        if query.is_empty() || query.len() > MAX_QUERY_BYTES || query.chars().any(char::is_control)
        {
            return Err("Each research query must contain 1–240 printable characters.".into());
        }
        if !unique
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(query))
        {
            unique.push(query.to_string());
        }
    }
    Ok(unique)
}

fn timestamp_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_secs()).ok())
        .unwrap_or_default()
}

fn public_ip(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(ip) => {
            let octets = ip.octets();
            !(ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_broadcast()
                || ip.is_unspecified()
                || ip.is_multicast()
                || octets[0] == 0
                || (octets[0] == 100 && (64..=127).contains(&octets[1]))
                || (octets[0] == 192 && octets[1] == 0)
                || (octets[0] == 192 && octets[1] == 0 && octets[2] == 2)
                || (octets[0] == 198 && (octets[1] == 18 || octets[1] == 19))
                || (octets[0] == 198 && octets[1] == 51 && octets[2] == 100)
                || (octets[0] == 203 && octets[1] == 0 && octets[2] == 113)
                || octets[0] >= 224)
        }
        IpAddr::V6(ip) => {
            !(ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_multicast()
                || ip.is_unique_local()
                || ip.is_unicast_link_local()
                || ip.to_ipv4_mapped().is_some())
        }
    }
}

fn validate_page_url(value: &str) -> Result<Url, String> {
    if value.len() > 2048 {
        return Err("Source URL exceeds the supported length.".into());
    }
    let mut url =
        Url::parse(value).map_err(|_| "Search result contains an invalid URL.".to_string())?;
    let host = url
        .host_str()
        .ok_or_else(|| "Search result does not contain a public hostname.".to_string())?;
    let host_lower = host.to_ascii_lowercase();
    let host_ip = host.trim_matches(['[', ']']).parse::<IpAddr>();
    if url.scheme() != "https"
        || url.port_or_known_default() != Some(443)
        || !url.username().is_empty()
        || url.password().is_some()
        || host_ip.is_ok()
        || host_lower == "localhost"
        || host_lower.ends_with(".localhost")
        || host_lower.ends_with(".local")
        || host_lower.ends_with(".internal")
        || host_lower.ends_with(".test")
    {
        return Err("Only public HTTPS web sources are allowed.".into());
    }
    url.set_fragment(None);
    Ok(url)
}

async fn resolve_public_host(host: String) -> Result<Vec<SocketAddr>, String> {
    let addresses = tauri::async_runtime::spawn_blocking(move || {
        (host.as_str(), 443)
            .to_socket_addrs()
            .map(|resolved| resolved.collect::<Vec<_>>())
    })
    .await
    .map_err(|_| "Could not safely resolve the source host.".to_string())?
    .map_err(|_| "Could not resolve the public source host.".to_string())?;
    if addresses.is_empty() || addresses.iter().any(|address| !public_ip(address.ip())) {
        return Err("The source host does not resolve exclusively to public IP addresses.".into());
    }
    Ok(addresses)
}

fn make_client() -> Result<Client, String> {
    Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .no_proxy()
        .redirect(Policy::none())
        .build()
        .map_err(|_| "The local web research client could not be initialized.".to_string())
}

async fn read_bounded(response: Response) -> Result<Vec<u8>, String> {
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return Err("A web response exceeded the 512 KiB size limit.".into());
    }
    let mut response = response;
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "A web response could not be read.".to_string())?
    {
        if body.len().saturating_add(chunk.len()) > MAX_RESPONSE_BYTES {
            return Err("A web response exceeded the 512 KiB size limit.".into());
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

async fn fetch_search_results(
    client: &Client,
    key: &str,
    engine: &str,
    query: &str,
) -> Result<Vec<GoogleItem>, String> {
    let response = client
        .get(GOOGLE_SEARCH_URL)
        .query(&[
            ("key", key),
            ("cx", engine),
            ("q", query),
            ("num", &MAX_CSE_ITEMS.to_string()),
            ("safe", "active"),
        ])
        .send()
        .await
        .map_err(|error| -> String {
            if error.is_timeout() {
                "Google search timed out. Check the connection and retry.".into()
            } else {
                "Could not connect to Google Custom Search.".into()
            }
        })?;
    if !response.status().is_success() {
        return Err(match response.status() {
            StatusCode::TOO_MANY_REQUESTS => {
                "Google Custom Search rate limit reached. Try again later.".into()
            }
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                "Google Custom Search rejected its local credentials or search-engine configuration.".into()
            }
            _ => "Google Custom Search returned an error. Verify its local configuration.".into(),
        });
    }
    let body = read_bounded(response).await?;
    let result: GoogleResponse = serde_json::from_slice(&body)
        .map_err(|_| "Google Custom Search returned an invalid response.".to_string())?;
    Ok(result.items.unwrap_or_default())
}

async fn retrieve_page(url: &Url) -> Result<String, String> {
    let host = url
        .host_str()
        .ok_or_else(|| "Source URL has no hostname.".to_string())?
        .to_string();
    let addresses = resolve_public_host(host.clone()).await?;
    let pinned_client = Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .no_proxy()
        .redirect(Policy::none())
        .resolve_to_addrs(&host, &addresses)
        .build()
        .map_err(|_| "Could not initialize the safe source reader.".to_string())?;
    let response = pinned_client
        .get(url.clone())
        .header(reqwest::header::ACCEPT, "text/html,application/xhtml+xml")
        .header(
            reqwest::header::USER_AGENT,
            "TREDALIX-AI/0.1 (public-source reader)",
        )
        .send()
        .await
        .map_err(|error| -> String {
            if error.is_timeout() {
                "Source page timed out.".into()
            } else {
                "Source page could not be retrieved.".into()
            }
        })?;
    if !response.status().is_success() {
        return Err(if response.status().is_redirection() {
            "Source page redirected; redirects are not followed for safety.".into()
        } else {
            format!("Source page returned HTTP {}.", response.status().as_u16())
        });
    }
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !content_type.contains("text/html") && !content_type.contains("application/xhtml+xml") {
        return Err("Source is not an accessible HTML page.".into());
    }
    let bytes = read_bounded(response).await?;
    String::from_utf8(bytes).map_err(|_| "Source page is not valid UTF-8 HTML.".to_string())
}

pub async fn search(queries: Vec<String>) -> Result<ResearchResult, String> {
    let queries = validate_queries(&queries)?;
    let provider = GoogleCseProvider::from_environment().ok_or_else(|| {
        "Web Search Not Configured. Configure a supported provider locally before searching."
            .to_string()
    })?;
    search_with_provider(&provider, queries).await
}

async fn search_with_provider<P: SearchProvider>(
    provider: &P,
    queries: Vec<String>,
) -> Result<ResearchResult, String> {
    let items = provider.search(&queries).await?;
    let mut result = ResearchResult {
        provider: provider.id(),
        queries: queries.clone(),
        sources: Vec::new(),
        pages_read: 0,
        pages_unavailable: 0,
        retrieved_at_utc: timestamp_now(),
    };
    for item in items {
        let Ok(url) = validate_page_url(&item.url) else {
            continue;
        };
        if result
            .sources
            .iter()
            .any(|source| source.url == url.as_str())
        {
            continue;
        }
        let retrieved_at_utc = timestamp_now();
        let (excerpt_html, retrieval_status) = match retrieve_page(&url).await {
            Ok(html) => {
                result.pages_read += 1;
                (Some(html), "page_read")
            }
            Err(_) => {
                result.pages_unavailable += 1;
                (None, "snippet_only")
            }
        };
        result.sources.push(ResearchSource {
            id: format!("source-{}", result.sources.len() + 1),
            title: item.title.chars().take(500).collect(),
            url: url.to_string(),
            snippet: item.snippet.chars().take(1200).collect(),
            excerpt_html,
            retrieved_at_utc,
            retrieval_status,
        });
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{MAX_QUERIES, public_ip, status_for, validate_page_url, validate_queries};
    use std::net::IpAddr;

    #[test]
    fn query_validation_deduplicates_and_bounds_search_budget() {
        assert_eq!(
            validate_queries(&["TREDALIX".into(), "tredalix".into(), "AI".into()]).unwrap(),
            ["TREDALIX", "AI"]
        );
        assert!(validate_queries(&[]).is_err());
        assert!(validate_queries(&vec!["x".into(); MAX_QUERIES + 1]).is_err());
        assert!(validate_queries(&["x".repeat(241)]).is_err());
        assert!(validate_queries(&["latest\nnews".into()]).is_err());
    }

    #[test]
    fn missing_provider_is_reported_without_claiming_search_is_available() {
        let status = status_for(false);
        assert!(!status.configured);
        assert_eq!(status.provider, "not_configured");
        assert!(status.notice.contains("Web Search Not Configured"));
        assert!(status.notice.contains("local Gemma chat is independent"));
    }

    #[test]
    fn configured_provider_status_does_not_expose_credentials() {
        let status = status_for(true);
        assert!(status.configured);
        assert_eq!(status.provider, "google_cse");
        assert!(status.notice.contains("credentials have not been tested"));
        assert!(!status.notice.contains("API_KEY"));
        assert!(!status.notice.contains("CX"));
    }

    #[test]
    fn source_urls_must_be_https_and_cannot_target_private_hosts() {
        for url in [
            "http://example.com/",
            "https://localhost/",
            "https://printer.local/",
            "https://127.0.0.1/",
            "https://[::1]/",
            "https://user@example.com/",
        ] {
            assert!(validate_page_url(url).is_err(), "{url}");
        }
        assert!(validate_page_url("https://example.com/news").is_ok());
    }

    #[test]
    fn public_address_check_blocks_private_and_documentation_ranges() {
        for address in [
            "127.0.0.1",
            "10.1.2.3",
            "172.20.1.1",
            "192.168.1.1",
            "169.254.1.1",
            "100.64.0.1",
            "192.0.2.1",
            "::1",
            "fc00::1",
            "fe80::1",
        ] {
            assert!(!public_ip(address.parse::<IpAddr>().unwrap()), "{address}");
        }
        assert!(public_ip("8.8.8.8".parse().unwrap()));
        assert!(public_ip("2606:4700:4700::1111".parse().unwrap()));
    }
}
