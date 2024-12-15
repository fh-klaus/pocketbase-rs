use serde_json::Value;

use crate::{
    collection::Collection,
    pocketbase::{AuthStore, AuthStoreModel},
    AuthClientResponseData, AuthenticationError, Credentials, ErrorResponse,
};

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
