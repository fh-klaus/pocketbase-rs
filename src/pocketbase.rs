use reqwest::{multipart::Form, RequestBuilder};
use serde::{Deserialize, Serialize};

/// A `PocketBase` Client. You can use it to send requests to the `PocketBase` instance.
///
/// # Example
///
/// ```ignore,rust
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
/// async fn main() -> Result<(), AuthenticationError> {
///     let mut pb = PocketBaseAdminBuilder::new("http://localhost:8081")
///         .auth_with_password("user@domain.com", "abc123")
///         .await?;
///
///     let request = pb.collection("test").get_one::<Test>("record_id").call().await;
///
///     dbg!(request);
///
///     Ok(())
/// }
#[derive(Clone, Debug)]
pub struct PocketBase {
    pub(crate) base_url: String,
    pub(crate) auth_store: Option<AuthStore>,
    pub(crate) reqwest_client: reqwest::Client,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuthStore {
    pub record: AuthStoreRecord,
    pub token: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStoreRecord {
    pub id: String,
    collection_id: String,
    collection_name: String,
    pub created: String,
    pub updated: String,
    email: String,
    email_visibility: bool,
    verified: bool,
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
