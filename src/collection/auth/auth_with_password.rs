use serde_json::Value;
use thiserror::Error;

use crate::{
    collection::Collection,
    pocketbase::{AuthStore, AuthStoreModel},
    AuthClientResponseData, Credentials, ErrorResponse,
};

/// Represents errors that can occur during the authentication process with the `PocketBase` API.
///
/// This enum defines various error types that may arise when attempting to authenticate,
/// each providing details about the specific issue encountered.
#[derive(Error, Debug)]
pub enum AuthenticationError {
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [400 Bad Request]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/400") HTTP error response.
    ///
    /// Tip: The credentials you provided may be incorrect.
    #[error("Authentication failed: Invalid Credentials. Given email and/or password is wrong.")]
    InvalidCredentials,
    /// Email and/or Password cannot be empty.
    ///
    /// This variant indicates that certain fields in the authentication request need to be validated.
    /// The fields are represented as booleans:
    ///
    /// - `identity`: is blank and shouldn't be.
    /// - `password`: is blank and shouldn't be.
    #[error(
        "Authentication failed: Empty Credential Field. Given email and/or password is empty."
    )]
    EmptyField {
        /// Is identity blank.
        identity: bool,
        /// Is password blank.
        password: bool,
    },
    /// The provided identity must be an email address.
    ///
    /// This variant indicates that the authentication request failed because the provided identity
    /// does not conform to the expected email format. The `PocketBase` API requires the identity to
    /// be a valid email address for authentication.
    #[error("Authentication failed. Given identity is not a valid email.")]
    IdentityMustBeEmail,
    /// An HTTP error occurred while communicating with the `PocketBase` API.
    ///
    /// This variant wraps a [`reqwest::Error`] and indicates that the request could not be completed
    /// due to network issues, invalid URL, timeouts, etc.
    #[error("Authentication failed. Couldn't reach the PocketBase API: {0}")]
    HttpError(reqwest::Error),
    /// When something unexpected was returned by the `PocketBase` REST API.
    ///
    /// Would usually mean that there is an error somewhere in this API wrapper.
    #[error("Authentication failed due to an unexpected response. Usually means a problem in the PocketBase API's wrapper.")]
    UnexpectedResponse,
    /// Occurs when you try to authenticate a `PocketBase` client without providing the collection name.
    #[error("Authentication failed due to missing collection name. [Example: PocketBaseClientBuilder::new(\"\")")]
    MissingCollection,
}

impl From<reqwest::Error> for AuthenticationError {
    fn from(error: reqwest::Error) -> Self {
        Self::HttpError(error)
    }
}

impl<'a> Collection<'a> {
    /// Authenticates a Client user with the `PocketBase` server using their email and password.
    ///
    /// # Parameters
    ///
    /// * `identity`: The **username** or **email** of the Client record to authenticate.
    /// * `password`: The auth record password.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `PocketBase` instance on success, which includes the authentication token for
    /// further authorized requests. On failure, it returns an `AuthenticationError`.
    ///
    /// # Errors
    ///
    /// This function will return an `AuthenticationError` if:
    ///
    /// - There is an HTTP-related error when making the request.
    /// - The response from the `PocketBase` instance cannot be parsed correctly.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::{error::Error, fs};
    ///
    /// use pocketbase_rs::PocketBase;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn Error>> {
    ///     let mut pb = PocketBase::new("http://localhost:8090");
    ///
    ///     let auth_data = pb.collection("users")
    ///         .auth_with_password("test@domain.com", "secure-password")
    ///         .await?;
    ///
    ///     println!("Auth Data: {auth_data:?}");
    /// }
    /// ```
    pub async fn auth_with_password(
        &mut self,
        identity: &str,
        password: &str,
    ) -> Result<AuthStore, AuthenticationError> {
        let uri = format!(
            "{}/api/collections/{}/auth-with-password",
            self.client.base_url, self.name
        );

        let credentials = Credentials { identity, password };

        let response = self
            .client
            .request_post_json(&uri, &credentials)
            .send()
            .await?;

        if response.status().is_success() {
            let json = response.json::<AuthClientResponseData>().await?;

            let auth_store = AuthStore {
                token: json.token,
                model: AuthStoreModel {
                    id: json.record.id,
                    created: json.record.created,
                    updated: json.record.updated,
                    email: json.record.email,
                },
            };

            self.client.update_auth_store(auth_store.clone());

            return Ok(auth_store);
        } else if response.status() == reqwest::StatusCode::BAD_REQUEST {
            let error_response: ErrorResponse =
                response.json().await.unwrap_or_else(|_| ErrorResponse {
                    code: 400,
                    message: "Unknown error".to_string(),
                    data: None,
                });

            if let Some(ref data) = error_response.data {
                // {
                //     "code": 400,
                //     "message": "Failed to authenticate.",
                //     "data": {}
                // }
                if data.as_object().is_some_and(serde_json::Map::is_empty) {
                    return Err(AuthenticationError::InvalidCredentials);
                }

                // Check for specific field validation errors
                let identity_error = data
                    .get("identity")
                    .and_then(|v| v.get("code").and_then(Value::as_str));

                match identity_error {
                    // {
                    //     "code": 400,
                    //     "message": "Something went wrong while processing your request.",
                    //     "data": {
                    //       "identity": {
                    //         "code": "validation_is_email",
                    //         "message": "Must be a valid email address."
                    //       }
                    //     }
                    // }
                    Some("validation_is_email") => {
                        return Err(AuthenticationError::IdentityMustBeEmail)
                    }

                    // {
                    //     "code": 400,
                    //     "message": "Something went wrong while processing your request.",
                    //     "data": {
                    //       "identity": {
                    //         "code": "validation_required",
                    //         "message": "Cannot be blank."
                    //       },
                    //       "password": {
                    //         "code": "validation_required",
                    //         "message": "Cannot be blank."
                    //       }
                    //     }
                    // }
                    Some("validation_required") => {
                        return Err(AuthenticationError::EmptyField {
                            identity: identity_error.is_some(),
                            password: data.get("password").is_some(),
                        })
                    }
                    None => {
                        return Err(AuthenticationError::EmptyField {
                            identity: false,
                            password: data.get("password").is_some(),
                        });
                    }
                    _ => {}
                }
            }

            // {
            //     "code": 400,
            //     "message": "Failed to authenticate.",
            //     "data": {}
            // }
            return Err(AuthenticationError::InvalidCredentials);
        }

        Err(AuthenticationError::UnexpectedResponse)
    }
}
