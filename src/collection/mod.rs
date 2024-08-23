use serde::Deserialize;

use crate::pocketbase::PocketBase;

mod auth;
mod create;
mod get_first_list_item;
mod get_full_list;
mod get_list;
mod get_one;
mod update;

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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordList<T> {
    pub page: i32,
    pub per_page: i32,
    pub total_items: i32,
    pub total_pages: i32,
    pub items: Vec<T>,
}
