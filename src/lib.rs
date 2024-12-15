#![deny(missing_docs)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(dead_code)]

//! pocketbase-rs is an open-source Rust wrapper around `PocketBase`'s REST API.
//!
//! # Usage
//!
//!  ```rust,ignore
//!  use std::error::Error;
//!
//!  use pocketbase_rs::{PocketBaseAdminBuilder, Collection, RequestError};
//!  use serde::Deserialize;
//!
//!  #[derive(Default, Deserialize, Clone)]
//!  struct Article {
//!      title: String,
//!      content: String,
//!  }
//!
//!  #[tokio::main]
//!  async fn main() -> Result<(), Box<dyn Error>> {
//!      let mut pb = PocketBaseAdminBuilder::new("http://localhost:8090".to_string());
//!             .auth_with_password("test@domain.com", "secure-password")
//!             .await?;
//!
//!      let request: Article = pb.collection("articles").get_one::<Article>("record_id_123").call().await;
//!
//!      match(request) {
//!          Ok(article) => println!("Article Title: {}", article.title);
//!          Err(error) => eprintln!("Error: {}", error);
//!      }
//!
//!      Ok(())
//!  }
//!  ```

pub use error::{BadRequestError, RequestError};
pub use pocketbase::PocketBase;
pub use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};

pub(crate) mod collection;
pub(crate) mod error;
pub(crate) mod pocketbase;

#[derive(Clone, Default, Serialize)]
struct Credentials<'a> {
    pub(crate) identity: &'a str,
    pub(crate) password: &'a str,
}

#[derive(Deserialize)]
pub(crate) struct AuthClientResponseData {
    record: AuthClientResponseDataRecord,
    token: String,
}

#[derive(Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthClientResponseDataRecord {
    pub collection_id: String,
    pub collection_name: String,
    pub created: String,
    pub email: String,
    pub email_visibility: bool,
    pub id: String,
    pub updated: String,
    pub verified: bool,
}

#[derive(Deserialize)]
pub(crate) struct AuthAdminResponseData {
    token: String,
    admin: AuthAdminResponseDataRecord,
}

#[derive(Deserialize)]
pub(crate) struct AuthAdminResponseDataRecord {
    pub id: String,
    pub created: String,
    pub updated: String,
    pub email: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    code: u16,
    message: String,
    data: Option<serde_json::Value>,
}
