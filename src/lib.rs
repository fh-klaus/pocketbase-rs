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

#![deny(missing_docs)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(dead_code)]

pub use error::{BadRequestError, RequestError};
pub use records::auth::{AuthStore, AuthStoreRecord};
pub use reqwest::multipart::{Form, Part};
use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};

pub(crate) mod error;
pub(crate) mod records;

/// Represents a specific collection in a `PocketBase` database.
///
/// The `Collection` struct provides an interface for interacting with a specific collection
/// within a `PocketBase` instance. Instances of this struct are created using the
/// [`PocketBase::collection`] method. All operations on the target collection, such as retrieving,
/// creating, updating, or deleting records, are accessible through methods implemented on
/// this struct.
///
/// # Fields
/// - `client`: A mutable reference to the `PocketBase` client instance.
///   This allows the `Collection` to send requests to the `PocketBase` server.
/// - `name`: The name of the collection being interacted with.
pub struct Collection<'a> {
    pub(crate) client: &'a mut PocketBase,
    pub(crate) name: &'a str,
}

impl PocketBase {
    /// Creates a new [`Collection`] instance for the specified collection name.
    ///
    /// This method provides access to operations related to a specific collection in the `PocketBase` server.
    /// Most interactions with the `PocketBase` API are performed through the [`Collection`] instance returned
    /// by this method.
    ///
    /// # Arguments
    /// * `collection_name` - The name of the collection to interact with, provided as a static string.
    ///
    /// # Returns
    /// A [`Collection`] instance configured for the specified collection.
    ///
    /// # Example
    ///
    /// ```
    /// let mut client = PocketBase::new("http://localhost:8090");
    ///
    /// let collection = client.auth_with_password("use@domain.com", "super-secure-password");
    ///
    /// let request = pb
    ///     .collection("articles")
    ///     .get_first_list_item::<Article>()
    ///     .filter("language='en'")
    ///     .call()
    ///     .await;
    /// ```
    pub const fn collection(&mut self, collection_name: &'static str) -> Collection {
        Collection {
            client: self,
            name: collection_name,
        }
    }
}

/// Represents a paginated list of records retrieved from a `PocketBase` collection.
///
/// The `RecordList` struct encapsulates the results of a paginated query to a collection.
/// It contains metadata about the pagination state (such as the current page, total items,
/// and total pages) as well as the records themselves.
///
/// This struct is typically returned by methods that fetch a list of records from a
/// collection, such as [`Collection::get_list`].
///
/// # Type Parameters
/// - `T`: The type of the records contained in the `items` list. This is typically a
///   deserialized struct that matches the schema of the records in the collection.
///
/// # Fields
/// - `page`: The current page number (starting from 1).
/// - `per_page`: The maximum number of records returned per page (default is 30).
/// - `total_items`: The total number of records in the collection that match the query.
/// - `total_pages`: The total number of pages available for the query.
/// - `items`: A vector containing the records for the current page.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordList<T> {
    /// The page (aka. offset) of the paginated list *(default to 1)*.
    pub page: i32,
    /// The max returned records per page *(default to 30)*.
    pub per_page: i32,
    /// The total amount of records found in the collection.
    pub total_items: i32,
    /// The total amount of pages found in the collection.
    pub total_pages: i32,
    /// A list of all records for the given page.
    pub items: Vec<T>,
}

#[derive(Deserialize)]
struct ErrorResponse {
    code: u16,
    message: String,
    data: Option<serde_json::Value>,
}

/// A `PocketBase` Client. You can use it to send requests to the `PocketBase` instance.
///
/// # Example
///
/// ```ignore,rust
/// use std::error::Error;
///
/// use pocketbase_rs::{AuthenticationError, PocketBase};
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// pub struct Test {
///     test: String,
/// }
///
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn Error>> {
///     let mut pb = PocketBaseAdminBuilder::new("http://localhost:8090");
///
///     // ...
///
///     let request = pb
///         .collection("test")
///         .get_one::<Test>("record_id")
///         .call()
///         .await?;
///
///     println!("Test: {test:?}");
///
///     Ok(())
/// }
#[derive(Clone, Debug)]
pub struct PocketBase {
    pub(crate) base_url: String,
    pub(crate) auth_store: Option<AuthStore>,
    pub(crate) reqwest_client: reqwest::Client,
}

impl PocketBase {
    /// Creates a new instance of the `PocketBase` client.
    ///
    /// This method initializes a new client that can be used to interact with a `PocketBase`
    /// instance can then be used to authenticate users, manage records, and perform other API operations.
    ///
    /// # Arguments
    /// * `base_url` - A string slice representing the base URL of the `PocketBase` instance (e.g., `"http://localhost:8090"`).
    ///
    /// # Returns
    /// A `PocketBase` client instance that can be used to make requests to the `PocketBase` server.
    ///
    /// # Example
    /// ```rust
    /// let client = PocketBase::new("http://localhost:8090");
    /// // Use the client for further operations like authentication or fetching records
    /// ```
    #[must_use]
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            auth_store: None,
            reqwest_client: reqwest::Client::new(),
        }
    }

    /// Retrieves the current auth store, if available.
    ///
    /// If the `PocketBase` client has an active authentication session, this method
    /// returns the authentication data stored in the client. Otherwise, it returns `None`.
    ///
    /// # Returns
    ///
    /// An `Option<AuthStore>` containing the authentication token if authenticated, or `None` if
    /// the client is not authenticated.
    ///
    /// # Example
    ///
    /// ```
    /// let client = PocketBase::new("http://localhost:8090");
    ///
    /// // ...
    ///
    /// if let Some(auth_store) = client.auth_store() {
    ///     println!("Authenticated with token: {}", auth_store.token);
    /// } else {
    ///     println!("Not authenticated");
    /// }
    /// ```
    #[must_use]
    pub fn auth_store(&self) -> Option<AuthStore> {
        self.auth_store.clone()
    }

    /// Retrieves the current authentication token, if available.
    ///
    /// If the `PocketBase` client has an active authentication session, this method
    /// returns the authentication token stored in the `auth_store`. Otherwise, it returns `None`.
    ///
    /// # Returns
    /// An `Option<String>` containing the authentication token if authenticated, or `None` if
    /// the client is not authenticated.
    ///
    /// # Example
    ///
    /// ```
    /// let pb = PocketBase::new("http://localhost:8090");
    ///
    /// // ...
    ///
    /// if let Some(token) = pb.token() {
    ///     println!("Authenticated with token: {}", token);
    /// } else {
    ///     println!("Not authenticated");
    /// }
    /// ```
    #[must_use]
    pub fn token(&self) -> Option<String> {
        self.auth_store
            .as_ref()
            .map(|auth_store| auth_store.token.clone())
    }

    /// Returns the base URL of the `PocketBase` server.
    ///
    /// This method retrieves the base URL that was set when the `PocketBase` client
    /// instance was created. The URL is typically used internally for making API requests to the instance.
    ///
    /// # Returns
    /// A `String` containing the base URL of the `PocketBase` instance.
    ///
    /// # Example
    ///
    /// ```
    /// let client = PocketBase::new("http://localhost:8090");
    /// assert_eq!(client.base_url(), "http://localhost:8090".to_string());
    /// ```
    #[must_use]
    pub fn base_url(&self) -> String {
        self.base_url.clone()
    }

    pub(crate) fn update_auth_store(&mut self, new_auth_store: AuthStore) {
        self.auth_store = Some(new_auth_store);
    }
}

impl PocketBase {
    /// Adds an authorization token to the request, if available.
    ///
    /// This method attaches a bearer authentication token to the provided `RequestBuilder`
    /// if the client is currently authenticated. If no token is available, the request is
    /// returned unchanged.
    ///
    /// # Arguments
    /// * `request_builder` - A `reqwest::RequestBuilder` to which the token will be added.
    ///
    /// # Returns
    /// A `reqwest::RequestBuilder` with the authorization token, if applicable.
    pub(crate) fn with_authorization_token(
        &self,
        request_builder: reqwest::RequestBuilder,
    ) -> reqwest::RequestBuilder {
        if let Some(auth_store) = self.auth_store() {
            request_builder.bearer_auth(auth_store.token)
        } else {
            request_builder
        }
    }

    /// Creates a POST request builder for the specified endpoint.
    ///
    /// This method initializes a `POST` request to the given endpoint and adds
    /// an authorization token if available.
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint to send the `POST` request to.
    ///
    /// # Returns
    /// A `reqwest::RequestBuilder` for the `POST` request.
    pub(crate) fn request_post(&self, endpoint: &str) -> RequestBuilder {
        let request_builder = self.reqwest_client.post(endpoint);
        self.with_authorization_token(request_builder)
    }

    /// Creates a PATCH request builder with JSON body for the specified endpoint.
    ///
    /// This method initializes a `PATCH` request to the given endpoint with a JSON body,
    /// and adds an authorization token if available.
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint to send the `PATCH` request to.
    /// * `params` - A reference to a serializable type to use as the JSON body of the request.
    ///
    /// # Returns
    /// A `reqwest::RequestBuilder` for the `PATCH` request.
    pub(crate) fn request_patch_json<T: Default + Serialize + Clone + Send>(
        &self,
        endpoint: &str,
        params: &T,
    ) -> RequestBuilder {
        let request_builder = self.reqwest_client.patch(endpoint).json(&params);
        self.with_authorization_token(request_builder)
    }

    /// Creates a POST request builder with JSON body for the specified endpoint.
    ///
    /// This method initializes a `POST` request to the given endpoint with a JSON body,
    /// and adds an authorization token if available.
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint to send the `POST` request to.
    /// * `params` - A reference to a serializable type to use as the JSON body of the request.
    ///
    /// # Returns
    /// A `reqwest::RequestBuilder` for the `POST` request.
    pub(crate) fn request_post_json<T: Default + Serialize + Clone + Send>(
        &self,
        endpoint: &str,
        params: &T,
    ) -> RequestBuilder {
        let request_builder = self.reqwest_client.post(endpoint).json(&params);
        self.with_authorization_token(request_builder)
    }

    /// Creates a POST request builder with a form body for the specified endpoint.
    ///
    /// This method initializes a `POST` request to the given endpoint with a multipart form body,
    /// and adds an authorization token if available.
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint to send the `POST` request to.
    /// * `form` - A `reqwest::multipart::Form` representing the form data for the request.
    ///
    /// # Returns
    /// A `reqwest::RequestBuilder` for the `POST` request.
    pub(crate) fn request_post_form(&self, endpoint: &str, form: Form) -> RequestBuilder {
        let request_builder = self.reqwest_client.post(endpoint).multipart(form);
        self.with_authorization_token(request_builder)
    }

    /// Creates a GET request builder for the specified endpoint.
    ///
    /// This method initializes a `GET` request to the given endpoint, adds an `Accept` header
    /// for JSON responses, attaches query parameters if provided, and adds an authorization
    /// token if available.
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint to send the `GET` request to.
    /// * `params` - An optional vector of key-value pairs to include as query parameters.
    ///
    /// # Returns
    /// A `reqwest::RequestBuilder` for the `GET` request.
    pub(crate) fn request_get(
        &self,
        endpoint: &str,
        params: Option<Vec<(&str, &str)>>,
    ) -> RequestBuilder {
        let mut request_builder = self
            .reqwest_client
            .get(endpoint)
            .header("Accept", "application/json");

        if let Some(params) = params {
            request_builder = request_builder.query(&params);
        }

        self.with_authorization_token(request_builder)
    }
}
