use std::collections::HashMap;

use crate::{collection::Collection, RequestError};

impl<'a> Collection<'a> {
    /// Request verification.
    ///
    /// Sends the user a verification email request.
    ///
    /// # Parameters
    ///
    /// * `email`: The email address of the user you wish to request verification from. (Example: user@domain.com)
    ///
    /// # Returns
    ///
    /// On success, this function returns an empty tuple. If an error occurs, it returns a `RequestError`, which may include:
    ///
    /// # Errors
    ///
    /// This function may return:
    /// - `RequestError::Forbidden` if the operation is not permitted.
    /// - `RequestError::NotFound` if the method is not available for the given collection. You probably made a mistake in the collection name, or the collection is not of type "Auth collection".
    /// - `RequestError::Unhandled` for all other error cases.
    ///
    /// # Example
    /// 
    /// ```rust,ignore
    /// use std::error::Error;
    /// 
    /// use pocketbase_rs::{AuthenticationError, PocketBaseAdminBuilder};
    /// use serde::{Deserialize, Serialize};
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn Error>> {
    ///     let pb = PocketBaseAdminBuilder::new("http://localhost:8081")
    ///         .auth_with_password("admin@domain.com", "secure-password")
    ///         .await?;
    ///
    ///     let refreshed_auth = pb
    ///         .collection("users")
    ///         .request_verification("user@domain.com")
    ///         .await;
    ///
    ///     match refreshed_auth {
    ///         Ok(_) => println!("The verification request was sent successfully."),
    ///         Err(error) => eprintln!("Failed to send verification request: {}", error),
    ///     }
    ///
    ///     Ok(())
    /// }
    /// }
    /// ```
    pub async fn request_verification(&self, email: &'a str) -> Result<(), RequestError> {
        let url = format!(
            "{}/api/collections/{}/request-verification",
            self.client.base_url, self.name
        );

        let email: HashMap<String, String> = HashMap::from([("email".to_string(), email.into())]);

        let request = (self.client.request_post_json(&url, &email)).send().await;

        match request {
            Ok(response) => match response.status() {
                reqwest::StatusCode::NO_CONTENT => Ok(()),
                reqwest::StatusCode::BAD_REQUEST => Err(RequestError::BadRequest(String::new())),
                reqwest::StatusCode::NOT_FOUND => Err(RequestError::NotFound),
                _ => Err(RequestError::Unhandled),
            },
            Err(error) => {
                if let Some(error_status) = error.status() {
                    match error_status {
                        reqwest::StatusCode::UNAUTHORIZED => {
                            return Err(RequestError::Unauthorized)
                        }
                        reqwest::StatusCode::FORBIDDEN => {
                            return Err(RequestError::Forbidden);
                        }
                        reqwest::StatusCode::NOT_FOUND => {
                            return Err(RequestError::NotFound);
                        }
                        _ => return Err(RequestError::Unhandled),
                    }
                }

                Err(RequestError::Unhandled)
            }
        }
    }
}
