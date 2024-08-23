use core::fmt;
use std::collections::HashMap;

use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GetListError {
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [403 Forbidden]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/403") HTTP error response.
    ///
    /// The authenticated user may not have permissions for this interaction.
    #[error("Forbidden: The authenticated user may not have permissions for this interaction.")]
    Forbidden,
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [400 Bad Request]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/400") HTTP error response.
    ///
    /// One of the given arguments (sort, expand, filter..) is probably wrong.
    #[error("Bad Request: Something went wrong while processing your request. {0}")]
    BadRequest(String),
    /// An unexpected error occurred.
    #[error("Unhandled error. If you think it should be, please open an issue on Github. : {0}")]
    Unhandled(String),
}

#[derive(Error, Debug)]
pub enum CreateError {
    /// Represents generic errors that can occurs when interacting with the `PocketBase` API.
    #[error("Generic Error received when attempting record creation: {0}")]
    RequestError(RequestError),
    /// Bad Request. One or more fields couldn't be validated by `PocketBase`..
    #[error("Bad Request. One or more fields couldn't be validated by PocketBase: {0:?}")]
    BadRequest(Vec<BadRequestError>),
    /// An unexpected error occurred.
    #[error("Unhandled error. If you think it should be, please open an issue on Github. : {0}")]
    Unhandled(String),
}

#[derive(Error, Debug)]
pub enum UpdateError {
    /// Represents generic errors that can occurs when interacting with the `PocketBase` API.
    #[error("Generic Error received when attempting record creation: {0}")]
    RequestError(RequestError),
    /// Bad Request. One or more fields couldn't be validated by `PocketBase`..
    #[error("Bad Request. One or more fields couldn't be validated by PocketBase: {0:?}")]
    BadRequest(Vec<BadRequestError>),
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [403 Forbidden]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/403") HTTP error response.
    ///
    /// The authenticated user may not have permissions for this interaction.
    #[error("Forbidden: The authenticated user may not have permissions for this interaction.")]
    Forbidden,
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [404 Not Found]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404") HTTP error response.
    #[error("Not Found: The requested resource could not be found.")]
    NotFound,
    /// The `PocketBase` API interaction timed out. It may be offline.
    #[error(
        "Unreachable: The PocketBase API interaction timed out, or the service may be offline."
    )]
    Unreachable,
    /// The response could not be parsed into the expected data structure.
    #[error("Parse Error: Could not parse response into the expected data structure. It usually means that there is a missmatch between the provided Generic Type Parameter and your Collection definition. - {0}")]
    ParseError(String),
    /// Unhandled error.
    ///
    /// Usually emitted when something unexpected happened, and isn't handled correctly by this crate.
    #[error("Unhandled Error: An unexpected error occurred: {0}")]
    Unhandled(String),
}

#[derive(Deserialize, Debug)]
pub struct BadRequestResponse {
    pub code: u16,
    pub message: String,
    pub data: HashMap<String, BadRequestInternalError>,
}

/// Represents an instance of one of the errors that could be returned on a bad request.
///
/// This struct holds detailed information about a single validation error,
/// including the field name, error code, and a user-friendly message.
#[derive(Deserialize, Debug)]
pub struct BadRequestError {
    /// Name of the field.
    pub name: String,
    /// Error code.
    pub code: String,
    /// More details about the error.
    pub message: String,
}

impl fmt::Display for BadRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} {}", self.name, self.code, self.message)
    }
}

#[derive(Deserialize, Debug)]
pub struct BadRequestInternalError {
    pub code: String,
    pub message: String,
}

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

/// Represents errors when interacting with the `PocketBase` API.
///
/// This enum provides a set of error types that may occur during
/// API requests, each indicating a specific issue encountered.
#[derive(Error, Debug)]
pub enum RequestError {
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [400 Bad Request]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/400") HTTP error response.
    ///
    /// Your request may be missing fields or its content doesn't match what `PocketBase` expects to receive.
    #[error("Bad Request: Something went wrong while processing your request. {0}")]
    BadRequest(String),
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [401 Unauthorized]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401") HTTP error response.
    ///
    /// The request may require an Authorization Token.
    #[error("Unauthorized: The request may require an Authorization Token.")]
    Unauthorized,
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [403 Forbidden]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/403") HTTP error response.
    ///
    /// The authenticated user may not have permissions for this interaction.
    #[error("Forbidden: The authenticated user may not have permissions for this interaction.")]
    Forbidden,
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [404 Not Found]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404") HTTP error response.
    #[error("Not Found: The requested resource could not be found.")]
    NotFound,
    /// The response could not be parsed into the expected data structure.
    #[error("Parse Error: Could not parse response into the expected data structure. It usually means that there is a missmatch between the provided Generic Type Parameter and your Collection definition. - {0}")]
    ParseError(String),
    /// The `PocketBase` API interaction timed out. It may be offline.
    #[error(
        "Unreachable: The PocketBase API interaction timed out, or the service may be offline."
    )]
    Unreachable,
    /// Unhandled error.
    ///
    /// Usually emitted when something unexpected happened, and isn't handled correctly by this crate.
    #[error("Unhandled Error: An unexpected error occurred.")]
    Unhandled,
}
