use crate::{
    pocketbase::{AuthStore, AuthStoreModel},
    AuthClientResponseData, RequestError,
};

use crate::collection::Collection;

impl Collection<'_> {
    /// Returns a new auth response (token and record data) for an **already authenticated record**.
    ///
    /// On success, this function returns an `AuthStore` instance with the new token and updated
    /// user information. If an error occurs, it returns a `RequestError`, which may include:
    ///
    /// # Errors
    ///
    /// This function may return:
    /// - `RequestError::Unauthorized` if the provided token is invalid.
    /// - `RequestError::Forbidden` if the operation is not permitted.
    /// - `RequestError::NotFound` if the target user or session cannot be located.
    /// - `RequestError::Unhandled` for all other error cases.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::error::Error;
    ///
    /// use pocketbase_rs::PocketBase;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn Error>> {
    ///    let mut pb = PocketBase::new("http://localhost:8090");
    ///
    ///    let auth_data = pb
    ///        .collection("_superusers")
    ///        .auth_with_password("test@test.com", "abcdefghijkl")
    ///        .await?;
    ///
    ///    println!("pre auth data: {auth_data:?}");
    ///
    ///    let auth_data = pb.collection("_superusers").auth_refresh().await?;
    ///
    ///    println!("post auth data: {auth_data:?}");
    ///
    ///    Ok(())
    ///}
    ///
    /// ```
    pub async fn auth_refresh(&mut self) -> Result<AuthStore, RequestError> {
        let url = format!(
            "{}/api/collections/{}/auth-refresh",
            self.client.base_url(),
            self.name
        );

        let request = self
            .client
            .request_post(&url)
            .bearer_auth(self.client.token().unwrap_or_default())
            .send()
            .await;

        match request {
            Ok(response) => {
                if response.status().is_success() {
                    let auth_store = {
                        let json: AuthClientResponseData = response.json().await.map_err(|_| {
                            RequestError::ParseError(
                                "Couldn't parse auth refresh response.".to_string(),
                            )
                        })?;

                        AuthStore {
                            token: json.token,
                            model: AuthStoreModel {
                                id: json.record.id,
                                created: json.record.created,
                                updated: json.record.updated,
                                email: json.record.email,
                            },
                        }
                    };

                    self.client.update_auth_store(auth_store.clone());

                    return Ok(auth_store);
                }

                match response.status() {
                    reqwest::StatusCode::UNAUTHORIZED => Err(RequestError::Unauthorized),
                    reqwest::StatusCode::FORBIDDEN => Err(RequestError::Forbidden),
                    reqwest::StatusCode::NOT_FOUND => Err(RequestError::NotFound),
                    _ => Err(RequestError::Unhandled),
                }
            }
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
