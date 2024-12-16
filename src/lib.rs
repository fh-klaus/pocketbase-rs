#![deny(missing_docs)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(dead_code)]

//! pocketbase-rs is an open-source Rust wrapper around `PocketBase`'s REST API.
//!
//! # Usage
//!
//! ```rust,ignore
//! use std::error::Error;
//!
//! use pocketbase_rs::{PocketBaseAdminBuilder, Collection, RequestError};
//! use serde::Deserialize;
//!
//! #[derive(Default, Deserialize, Clone)]
//! struct Article {
//!     title: String,
//!     content: String,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     let mut pb = PocketBase::new("http://localhost:8090");
//! 
//!     let auth_data = pb
//!         .auth_with_password("test@domain.com", "secure-password")
//!         .await?;
//!
//!     let article: Article = pb
//!         .collection("articles")
//!         .get_one::<Article>("record_id_123")
//!         .call()
//!         .await?;
//!
//!     println!("Article Title: {}", article.title);
//! 
//!     Ok(())
//! }
//! ```

pub use error::{BadRequestError, RequestError};
pub use pocketbase::PocketBase;
pub use reqwest::multipart::{Form, Part};
use serde::Deserialize;

pub(crate) mod collection;
pub(crate) mod error;
pub(crate) mod pocketbase;

#[derive(Deserialize)]
struct ErrorResponse {
    code: u16,
    message: String,
    data: Option<serde_json::Value>,
}
