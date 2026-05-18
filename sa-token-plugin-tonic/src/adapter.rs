// Author: 金书记
//
//! Tonic gRPC request/response adapters.

use http::HeaderMap;
use sa_token_adapter::{SaRequest, SaResponse, CookieOptions};
use sa_token_adapter::utils::parse_cookies;
use serde::Serialize;
use std::collections::HashMap;

/// gRPC request adapter that wraps tonic metadata.
pub struct TonicRequestAdapter {
    headers: HashMap<String, String>,
    method: String,
    path: String,
}

impl TonicRequestAdapter {
    /// Create a new request adapter.
    pub fn new(headers: HashMap<String, String>, method: String, path: String) -> Self {
        Self {
            headers,
            method,
            path,
        }
    }

    /// Create from tonic metadata map.
    pub fn from_metadata(
        metadata: &tonic::metadata::MetadataMap,
        method: String,
        path: String,
    ) -> Self {
        let mut headers = HashMap::new();
        for item in metadata.iter() {
            match item {
                tonic::metadata::KeyAndValueRef::Ascii(key, value) => {
                    // Ascii values can be converted to str
                    headers.insert(key.to_string(), format!("{:?}", value));
                }
                tonic::metadata::KeyAndValueRef::Binary(key, value) => {
                    // Binary values are base64 encoded in Debug format
                    headers.insert(key.to_string(), format!("{:?}", value));
                }
            }
        }
        Self {
            headers,
            method,
            path,
        }
    }

    /// Create from http HeaderMap.
    pub fn from_header_map(headers: &HeaderMap, method: String, path: String) -> Self {
        let mut map = HashMap::new();
        for (key, value) in headers.iter() {
            let k = key.to_string();
            let v = value.to_str().unwrap_or("");
            map.insert(k, v.to_string());
        }
        Self {
            headers: map,
            method,
            path,
        }
    }

    /// Get all headers.
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Get header by name.
    pub fn get(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }
}

impl SaRequest for TonicRequestAdapter {
    fn get_header(&self, name: &str) -> Option<String> {
        self.headers.get(name).cloned()
    }

    fn get_cookie(&self, name: &str) -> Option<String> {
        self.get("cookie")
            .and_then(|cookies| parse_cookies(cookies).get(name).cloned())
    }

    fn get_param(&self, _name: &str) -> Option<String> {
        None
    }

    fn get_path(&self) -> String {
        self.path.clone()
    }

    fn get_method(&self) -> String {
        self.method.clone()
    }
}

/// Response wrapper for gRPC responses.
pub struct TonicResponseAdapter {
    status: u16,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

impl TonicResponseAdapter {
    /// Create a new response adapter.
    pub fn new() -> Self {
        Self {
            status: 200,
            headers: Vec::new(),
            body: None,
        }
    }

    /// Get the response status.
    pub fn status(&self) -> u16 {
        self.status
    }

    /// Get all headers.
    pub fn headers(&self) -> &[(String, String)] {
        &self.headers
    }

    /// Get the response body.
    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }
}

impl Default for TonicResponseAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl SaResponse for TonicResponseAdapter {
    fn set_header(&mut self, name: &str, value: &str) {
        self.headers.push((name.to_string(), value.to_string()));
    }

    fn set_cookie(&mut self, name: &str, value: &str, options: CookieOptions) {
        let mut cookie = format!("{}={}", name, value);

        if let Some(domain) = options.domain {
            cookie.push_str(&format!("; Domain={}", domain));
        }
        if let Some(path) = options.path {
            cookie.push_str(&format!("; Path={}", path));
        }
        if let Some(max_age) = options.max_age {
            cookie.push_str(&format!("; Max-Age={}", max_age));
        }
        if options.http_only {
            cookie.push_str("; HttpOnly");
        }
        if options.secure {
            cookie.push_str("; Secure");
        }

        self.set_header("Set-Cookie", &cookie);
    }

    fn set_status(&mut self, status: u16) {
        self.status = status;
    }

    fn set_json_body<U: Serialize>(&mut self, body: U) -> Result<(), serde_json::Error> {
        let json = serde_json::to_string(&body)?;
        self.body = Some(json);
        self.headers.push(("Content-Type".to_string(), "application/json".to_string()));
        Ok(())
    }
}
