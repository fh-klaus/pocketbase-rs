use serde::{Deserialize, Serialize};

use crate::{
    error::{BadRequestError, BadRequestResponse, CreateError},
    RequestError,
};

use super::Collection;

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateResponse {
    pub collection_name: String,
    pub collection_id: String,
    pub id: String,
    pub updated: String,
    pub created: String,
}

impl Collection<'_> {
    /// Create a new record in the given collection, from the given struct.
    ///
    /// If you need to upload files, you should look at [`Collection::create_multipart()`] instead.
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
    ///     let request = admin_pb
    ///         .collection("articles")
    ///         .create::<Article>(Article {
    ///             name: "test".to_string(),
    ///             content: "an interesting article content.".to_string(),
    ///         })
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
    pub async fn create<T: Default + Serialize + Clone + Send>(
        self,
        record: T,
    ) -> Result<CreateResponse, CreateError> {
        let collection_name = self.name;

        let endpoint = format!(
            "{}/api/collections/{}/records",
            self.client.base_url, collection_name
        );

        let request = self
            .client
            .request_post_json(&endpoint, &record)
            .send()
            .await;

        match request {
            Ok(response) => {
                if response.status().is_success() {
                    let data = response.json::<CreateResponse>().await;

                    match data {
                        Ok(data) => return Ok(data),
                        Err(error) => {
                            return Err(CreateError::RequestError(RequestError::ParseError(
                                error.to_string(),
                            )))
                        }
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

                                Err(CreateError::BadRequest(errors))
                            }
                            Err(error) => Err(CreateError::RequestError(RequestError::ParseError(
                                error.to_string(),
                            ))),
                        }
                    }
                    reqwest::StatusCode::FORBIDDEN => {
                        Err(CreateError::RequestError(RequestError::Forbidden))
                    }
                    reqwest::StatusCode::NOT_FOUND => {
                        Err(CreateError::RequestError(RequestError::NotFound))
                    }
                    _ => Err(CreateError::RequestError(RequestError::Unhandled)),
                }
            }

            Err(error) => {
                if let Some(error_status) = error.status() {
                    match error_status {
                        reqwest::StatusCode::FORBIDDEN => {
                            return Err(CreateError::RequestError(RequestError::Forbidden))
                        }
                        reqwest::StatusCode::NOT_FOUND => {
                            return Err(CreateError::RequestError(RequestError::NotFound))
                        }
                        _ => return Err(CreateError::RequestError(RequestError::Unhandled)),
                    }
                }

                Err(CreateError::RequestError(RequestError::Unhandled))
            }
        }
    }

    /// Create a new record in the given collection, from the given [`crate::Form`].
    ///
    /// If you don't need to upload files, you probably want the "simpler" [`Collection::create()`] instead.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::{error::Error, fs};
    ///
    /// use pocketbase_rs::{Form, Part, PocketBaseAdminBuilder};
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
    ///     let image = fs::read("./vulpes_vulpes.jpg")?;
    ///
    ///     let image_part = Part::bytes(image)
    ///         .file_name("vulpes_vulpes")
    ///         .mime_str("image/jpeg")?;
    ///
    ///     let form = Form::new()
    ///         .text("name", "Red Fox")
    ///         .part("illustration", image_part);
    ///
    ///     let request = admin_pb
    ///         .collection("foxes")
    ///         .create_multipart(form)
    ///         .await;
    ///
    ///     match request {
    ///         Ok(record) => println!("Ok: {:?}", record),
    ///         Err(error) => eprintln!("Error: {error}"),
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn create_multipart(
        self,
        form: reqwest::multipart::Form,
    ) -> Result<CreateResponse, CreateError> {
        let collection_name = self.name;

        let endpoint = format!(
            "{}/api/collections/{}/records",
            self.client.base_url, collection_name
        );

        let request = self.client.request_post_form(&endpoint, form).send().await;

        match request {
            Ok(response) => {
                if response.status().is_success() {
                    let data = response.json::<CreateResponse>().await;

                    match data {
                        Ok(data) => return Ok(data),
                        Err(error) => {
                            return Err(CreateError::RequestError(RequestError::ParseError(
                                error.to_string(),
                            )))
                        }
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

                                Err(CreateError::BadRequest(errors))
                            }
                            Err(error) => Err(CreateError::RequestError(RequestError::ParseError(
                                error.to_string(),
                            ))),
                        }
                    }
                    reqwest::StatusCode::FORBIDDEN => {
                        Err(CreateError::RequestError(RequestError::Forbidden))
                    }
                    reqwest::StatusCode::NOT_FOUND => {
                        Err(CreateError::RequestError(RequestError::NotFound))
                    }
                    _ => Err(CreateError::RequestError(RequestError::Unhandled)),
                }
            }

            Err(error) => {
                if let Some(error_status) = error.status() {
                    match error_status {
                        reqwest::StatusCode::FORBIDDEN => {
                            return Err(CreateError::RequestError(RequestError::Forbidden))
                        }
                        reqwest::StatusCode::NOT_FOUND => {
                            return Err(CreateError::RequestError(RequestError::NotFound))
                        }
                        _ => return Err(CreateError::RequestError(RequestError::Unhandled)),
                    }
                }

                Err(CreateError::RequestError(RequestError::Unhandled))
            }
        }
    }
}
