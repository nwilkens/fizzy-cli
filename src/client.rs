#![allow(dead_code)]

use anyhow::{anyhow, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use reqwest::{Client, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::config::Config;

pub struct FizzyClient {
    http: Client,
    base_url: String,
    account_slug: String,
    token: String,
}

impl FizzyClient {
    pub fn new(config: &Config, account_override: Option<&str>, url_override: Option<&str>) -> Result<Self> {
        let token = config.require_token()?;
        let base_url = url_override
            .map(|s| s.to_string())
            .unwrap_or_else(|| config.base_url());
        let account_slug = account_override
            .map(|s| s.to_string())
            .unwrap_or_else(|| config.account().unwrap_or_default());

        let http = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        Ok(Self {
            http,
            base_url,
            account_slug,
            token,
        })
    }

    /// Create a client without requiring account (for login/identity endpoints)
    pub fn new_unscoped(config: &Config, url_override: Option<&str>) -> Result<Self> {
        let token = config.require_token()?;
        let base_url = url_override
            .map(|s| s.to_string())
            .unwrap_or_else(|| config.base_url());

        let http = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        Ok(Self {
            http,
            base_url,
            account_slug: String::new(),
            token,
        })
    }

    /// Create a client with just a token (for login verification)
    pub fn with_token(base_url: &str, token: &str) -> Result<Self> {
        let http = Client::builder().build()?;
        Ok(Self {
            http,
            base_url: base_url.to_string(),
            account_slug: String::new(),
            token: token.to_string(),
        })
    }

    /// Create a client with no auth (for magic link flow)
    pub fn unauthenticated(base_url: &str) -> Result<Self> {
        let http = Client::builder().build()?;
        Ok(Self {
            http,
            base_url: base_url.to_string(),
            account_slug: String::new(),
            token: String::new(),
        })
    }

    fn default_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(USER_AGENT, HeaderValue::from_static("fz/0.1.0"));
        if !self.token.is_empty() {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", self.token)) {
                headers.insert(AUTHORIZATION, val);
            }
        }
        Ok::<_, anyhow::Error>(headers).unwrap_or_default()
    }

    fn account_url(&self, path: &str) -> String {
        format!("{}/{}{}", self.base_url, self.account_slug, path)
    }

    fn global_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn check_response(response: &Response) -> Result<()> {
        let status = response.status();
        if status.is_success() {
            return Ok(());
        }
        match status {
            StatusCode::UNAUTHORIZED => Err(anyhow!("Authentication failed. Run `fz login` to authenticate.")),
            StatusCode::FORBIDDEN => Err(anyhow!("Permission denied.")),
            StatusCode::NOT_FOUND => Err(anyhow!("Not found.")),
            StatusCode::UNPROCESSABLE_ENTITY => Err(anyhow!("Validation error.")),
            StatusCode::TOO_MANY_REQUESTS => Err(anyhow!("Rate limited. Try again later.")),
            _ => Err(anyhow!("Request failed with status {status}")),
        }
    }

    fn parse_link_next(headers: &HeaderMap) -> Option<String> {
        headers
            .get("link")
            .and_then(|v| v.to_str().ok())
            .and_then(|link| {
                link.split(',')
                    .find(|part| part.contains("rel=\"next\""))
                    .and_then(|part| {
                        let url = part
                            .trim()
                            .split(';')
                            .next()?
                            .trim()
                            .trim_start_matches('<')
                            .trim_end_matches('>');
                        Some(url.to_string())
                    })
            })
    }

    // --- GET (single resource) ---

    pub async fn get_global<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.global_url(path);
        let resp = self.http.get(&url).headers(self.default_headers()).send().await?;
        Self::check_response(&resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn get_global_raw(&self, path: &str) -> Result<serde_json::Value> {
        let url = self.global_url(path);
        let resp = self.http.get(&url).headers(self.default_headers()).send().await?;
        Self::check_response(&resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.account_url(path);
        let resp = self.http.get(&url).headers(self.default_headers()).send().await?;
        Self::check_response(&resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn get_raw(&self, path: &str) -> Result<serde_json::Value> {
        let url = self.account_url(path);
        let resp = self.http.get(&url).headers(self.default_headers()).send().await?;
        Self::check_response(&resp).await?;
        Ok(resp.json().await?)
    }

    // --- GET (paginated list) ---

    pub async fn get_list<T: DeserializeOwned>(&self, path: &str, fetch_all: bool) -> Result<Vec<T>> {
        let url = self.account_url(path);
        self.get_list_by_url(&url, fetch_all).await
    }

    pub async fn get_list_global<T: DeserializeOwned>(&self, path: &str, fetch_all: bool) -> Result<Vec<T>> {
        let url = self.global_url(path);
        self.get_list_by_url(&url, fetch_all).await
    }

    pub async fn get_list_raw(&self, path: &str, fetch_all: bool) -> Result<serde_json::Value> {
        let url = self.account_url(path);
        self.get_list_raw_by_url(&url, fetch_all).await
    }

    pub async fn get_list_global_raw(&self, path: &str, fetch_all: bool) -> Result<serde_json::Value> {
        let url = self.global_url(path);
        self.get_list_raw_by_url(&url, fetch_all).await
    }

    async fn get_list_by_url<T: DeserializeOwned>(&self, initial_url: &str, fetch_all: bool) -> Result<Vec<T>> {
        let mut all_items = Vec::new();
        let mut current_url = Some(initial_url.to_string());

        while let Some(url) = current_url {
            let resp = self.http.get(&url).headers(self.default_headers()).send().await?;
            Self::check_response(&resp).await?;

            let next = Self::parse_link_next(resp.headers());
            let items: Vec<T> = resp.json().await?;
            all_items.extend(items);

            current_url = if fetch_all { next } else { None };
        }

        Ok(all_items)
    }

    async fn get_list_raw_by_url(&self, initial_url: &str, fetch_all: bool) -> Result<serde_json::Value> {
        let mut all_items: Vec<serde_json::Value> = Vec::new();
        let mut current_url = Some(initial_url.to_string());

        while let Some(url) = current_url {
            let resp = self.http.get(&url).headers(self.default_headers()).send().await?;
            Self::check_response(&resp).await?;

            let next = Self::parse_link_next(resp.headers());
            let items: Vec<serde_json::Value> = resp.json().await?;
            all_items.extend(items);

            current_url = if fetch_all { next } else { None };
        }

        Ok(serde_json::Value::Array(all_items))
    }

    // --- POST ---

    pub async fn post<B: Serialize, T: DeserializeOwned>(&self, path: &str, body: &B) -> Result<T> {
        let url = self.account_url(path);
        let resp = self
            .http
            .post(&url)
            .headers(self.default_headers())
            .header(CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await?;
        Self::check_response(&resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn post_raw<B: Serialize>(&self, path: &str, body: &B) -> Result<serde_json::Value> {
        let url = self.account_url(path);
        let resp = self
            .http
            .post(&url)
            .headers(self.default_headers())
            .header(CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await?;
        Self::check_response(&resp).await?;
        // Some POST endpoints return 201 with Location header and no body or with body
        let text = resp.text().await?;
        if text.is_empty() {
            Ok(serde_json::Value::Null)
        } else {
            Ok(serde_json::from_str(&text)?)
        }
    }

    pub async fn post_no_body(&self, path: &str) -> Result<()> {
        let url = self.account_url(path);
        let resp = self
            .http
            .post(&url)
            .headers(self.default_headers())
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?;
        Self::check_response(&resp).await?;
        Ok(())
    }

    pub async fn post_global<B: Serialize, T: DeserializeOwned>(&self, path: &str, body: &B) -> Result<T> {
        let url = self.global_url(path);
        let resp = self
            .http
            .post(&url)
            .headers(self.default_headers())
            .header(CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await?;
        Self::check_response(&resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn post_global_raw<B: Serialize>(&self, path: &str, body: &B) -> Result<serde_json::Value> {
        let url = self.global_url(path);
        let resp = self
            .http
            .post(&url)
            .headers(self.default_headers())
            .header(CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await?;
        Self::check_response(&resp).await?;
        let text = resp.text().await?;
        if text.is_empty() {
            Ok(serde_json::Value::Null)
        } else {
            Ok(serde_json::from_str(&text)?)
        }
    }

    // --- PUT ---

    pub async fn put<B: Serialize>(&self, path: &str, body: &B) -> Result<()> {
        let url = self.account_url(path);
        let resp = self
            .http
            .put(&url)
            .headers(self.default_headers())
            .header(CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await?;
        Self::check_response(&resp).await?;
        Ok(())
    }

    pub async fn put_with_response<B: Serialize, T: DeserializeOwned>(&self, path: &str, body: &B) -> Result<T> {
        let url = self.account_url(path);
        let resp = self
            .http
            .put(&url)
            .headers(self.default_headers())
            .header(CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await?;
        Self::check_response(&resp).await?;
        Ok(resp.json().await?)
    }

    // --- PATCH ---

    pub async fn patch<B: Serialize>(&self, path: &str, body: &B) -> Result<()> {
        let url = self.account_url(path);
        let resp = self
            .http
            .patch(&url)
            .headers(self.default_headers())
            .header(CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await?;
        Self::check_response(&resp).await?;
        Ok(())
    }

    pub async fn patch_with_response<B: Serialize, T: DeserializeOwned>(&self, path: &str, body: &B) -> Result<T> {
        let url = self.account_url(path);
        let resp = self
            .http
            .patch(&url)
            .headers(self.default_headers())
            .header(CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await?;
        Self::check_response(&resp).await?;
        Ok(resp.json().await?)
    }

    // --- DELETE ---

    pub async fn delete(&self, path: &str) -> Result<()> {
        let url = self.account_url(path);
        let resp = self.http.delete(&url).headers(self.default_headers()).send().await?;
        Self::check_response(&resp).await?;
        Ok(())
    }

    // --- Magic link auth flow (unauthenticated) ---

    pub async fn request_magic_link(&self, email: &str) -> Result<String> {
        let url = self.global_url("/session");
        let body = crate::models::MagicLinkRequest {
            email_address: email.to_string(),
        };
        let resp = self
            .http
            .post(&url)
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .header(USER_AGENT, "fz/0.1.0")
            .json(&body)
            .send()
            .await?;
        Self::check_response(&resp).await?;
        let parsed: crate::models::PendingAuthResponse = resp.json().await?;
        Ok(parsed.pending_authentication_token)
    }

    pub async fn submit_magic_link_code(
        &self,
        code: &str,
        pending_token: &str,
    ) -> Result<String> {
        let url = self.global_url("/session/magic_link");
        let body = crate::models::MagicLinkCodeRequest {
            code: code.to_string(),
        };
        let resp = self
            .http
            .post(&url)
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .header(USER_AGENT, "fz/0.1.0")
            .header(
                "Cookie",
                format!("pending_authentication_token={pending_token}"),
            )
            .json(&body)
            .send()
            .await?;
        Self::check_response(&resp).await?;
        let parsed: crate::models::SessionResponse = resp.json().await?;
        Ok(parsed.session_token)
    }

    pub async fn create_access_token_with_session(
        &self,
        session_token: &str,
        account_slug: &str,
    ) -> Result<crate::models::AccessTokenResponse> {
        let url = format!("{}/{}/my/access_tokens", self.base_url, account_slug);
        let body = crate::models::CreateAccessTokenRequest {
            access_token: crate::models::CreateAccessTokenBody {
                description: "fz CLI".to_string(),
                permission: "write".to_string(),
            },
        };
        let resp = self
            .http
            .post(&url)
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .header(USER_AGENT, "fz/0.1.0")
            .header("Cookie", format!("session_token={session_token}"))
            .json(&body)
            .send()
            .await?;
        Self::check_response(&resp).await?;
        Ok(resp.json().await?)
    }

    // --- Helpers ---

    pub fn account_slug(&self) -> &str {
        &self.account_slug
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}
