#![feature(backtrace)]
#[macro_use]
extern crate serde_json;

use std::error::Error;
use std::fmt::Display;
use hyper::Method;
use hyper::Request;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::json;
use serde_json::Value;
use hyper::client::{Client as HyperClient, HttpConnector};
use hyper::body::Body;
use hyper_tls::HttpsConnector;

pub struct TwitchGqlClient {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub base_url: String,
    pub hyper: HyperClient<HttpsConnector<HttpConnector>>,
}

pub struct GqlRequestBuilder {
    requests: Vec<Value>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GqlError {
    pub error: String,
    pub message: String,
    pub status: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Extensions {
    #[serde(rename = "durationMilliseconds")]
    pub duration_milliseconds: i32,
    #[serde(rename = "operationName")]
    pub operation_name: String,
    #[serde(rename = "requestID")]
    pub request_id: String,

}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub extensions: Extensions,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Clip {
    pub id: String,
    #[serde(rename = "videoOffsetSeconds")]
    pub video_offset_seconds: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClipsFullVideoButton {
    pub clip: Clip,
}

impl Display for GqlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\nstatus: {}\n{}", self.error, self.status, self.message)
    }
}

impl Error for GqlError {

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn backtrace(&self) -> Option<&std::backtrace::Backtrace> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl GqlRequestBuilder {

    pub fn new() -> Self {
        GqlRequestBuilder {
            requests: Vec::new()
        }
    }

    pub fn clips_full_video_button<S: Into<String> + serde::ser::Serialize>(mut self, clip_slug: S) -> Self {
        self.requests.push(clips_full_video_button(clip_slug));
        self
    }
}

impl TwitchGqlClient {

    pub fn new_unauth<S: Into<String>>(client_id: S) -> Self {
        let https = HttpsConnector::new();
        let hyper = HyperClient::builder().build::<_, Body>(https);

        TwitchGqlClient {
            client_id: client_id.into(),
            client_secret: None,
            base_url: "https://gql.twitch.tv/gql".to_string(),
            hyper,
        }
    }

    pub async fn send_request(&self, request: GqlRequestBuilder) -> Result<Value, Box<dyn Error>> {

        let s = serde_json::to_string(&Value::Array(request.requests))?;

        let req = Request::builder()
            .method(Method::POST)
            .header("Client-Id", &self.client_id)
            .header("Content-Type", "text/plain;charset=UTF-8")
            .uri(self.base_url.clone())
            .body(Body::from(s))?;

        let res = self.hyper.request(req).await?;
        let (_head, body) = res.into_parts();
        let bytes = hyper::body::to_bytes(body).await?;
        let maybe_error = serde_json::from_slice::<GqlError>(bytes.as_ref());

        if let Ok(err) = maybe_error {
            return Err(Box::new(err));
        }

        let res = serde_json::from_slice(bytes.as_ref())?;
        Ok(res)
    }

}

pub fn clips_full_video_button<S: Into<String> + serde::ser::Serialize>(clip_slug: S) -> Value {
    json!({
            "operationName": "ClipsFullVideoButton",
            "variables": {
                "slug": clip_slug.into()
            },
            "extensions": {
                "persistedQuery": {
                    "sha256Hash": "d519a5a70419d97a3523be18fe6be81eeb93429e0a41c3baa9441fc3b1dffebf",
                    "version": 1
                }
            },
        })
}
