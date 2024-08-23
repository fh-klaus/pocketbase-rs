use serde::{Deserialize, Serialize};

use crate::{
    error::{BadRequestResponse, UpdateError},
    pocketbase::PocketBase,
    BadRequestError,
};

use super::Collection;

pub struct CollectionUpdateBuilder<'a, T: Send + Serialize + Deserialize<'a>> {
    client: &'a PocketBase,
    collection_name: &'a str,
    record_id: &'a str,
    data: T,
    _marker: std::marker::PhantomData<T>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResponse {
    pub collection_name: String,
    pub collection_id: String,
    pub id: String,
    pub updated: String,
    pub created: String,
}

impl<'a> Collection<'a> {
    /// Update a single record.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::error::Error;
    ///
    /// use pocketbase_rs::{AuthenticationError, PocketBaseAdminBuilder};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Default, Serialize, Deserialize, Clone, Debug)]
    /// pub struct Article {
    ///     name: String,
    ///     content: String,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn Error>> {
    ///     let mut admin_pb = PocketBaseAdminBuilder::new("http://localhost:8081")
    ///         .auth_with_password("test@test.com", "abcdefghijkl")
    ///         .await?;
    ///
    ///     let updated_article = Article {
    ///         name: String::from("Updated Article Title"),
    ///         content: String::from("Updated article content"),
    ///     };
    ///
    ///     let request = admin_pb
    ///         .collection("articles")
    ///         .update::<Article>("jla0s0s86d83wx8", updated_article)
    ///         .await;
    ///
    ///     match request {
    ///         Ok(article) => println!("Ok: {:?}", article),
    ///         Err(error) => eprintln!("Error: {error}"),
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn update<T: Default + Serialize + Clone + Send>(
        self,
        record_id: &'a str,
        record: T,
    ) -> Result<UpdateResponse, UpdateError> {
        let collection_name = self.name;

        let endpoint = format!(
            "{}/api/collections/{}/records/{}",
            self.client.base_url, collection_name, record_id
        );

        let request = self
            .client
            .request_patch_json(&endpoint, &record)
            .send()
            .await;

        match request {
            Ok(response) => {
                if response.status().is_success() {
                    let data = response.json::<UpdateResponse>().await;

                    match data {
                        Ok(data) => return Ok(data),
                        Err(error) => return Err(UpdateError::ParseError(error.to_string())),
                    }
                }

                match response.status() {
                    reqwest::StatusCode::BAD_REQUEST => {
                        let data = response.json::<BadRequestResponse>().await;

                        match data {
                            Ok(bad_response) => {
                                let mut errors: Vec<BadRequestError> = vec![];

                                for (error_name, error_data) in bad_response.data {
                                    errors.push(BadRequestError {
                                        name: error_name,
                                        code: error_data.code,
                                        message: error_data.message,
                                    });
                                }

                                Err(UpdateError::BadRequest(errors))
                            }
                            Err(error) => Err(UpdateError::ParseError(error.to_string())),
                        }
                    }
                    reqwest::StatusCode::FORBIDDEN => Err(UpdateError::Forbidden),
                    reqwest::StatusCode::NOT_FOUND => Err(UpdateError::NotFound),
                    _ => Err(UpdateError::Unhandled(response.status().to_string())),
                }
            }
            Err(error) => {
                if let Some(status_code) = error.status() {
                    match status_code {
                        reqwest::StatusCode::FORBIDDEN => return Err(UpdateError::Forbidden),
                        reqwest::StatusCode::NOT_FOUND => return Err(UpdateError::NotFound),
                        _ => return Err(UpdateError::Unhandled(status_code.to_string())),
                    }
                }

                Err(UpdateError::Unhandled(error.to_string()))
            }
        }
    }
}
