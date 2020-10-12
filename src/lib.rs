//! An async-session implementation for MongoDB
//!
//! # Examples
//!
//! ```
//! use async_mongodb_session::*;
//! use async_session::{Session, SessionStore};
//!
//! # fn main() -> async_session::Result { async_std::task::block_on(async {
//! let store = MongodbSessionStore::connect("mongodb://127.0.0.1:27017", "db_name", "collection");
//! # Ok(()) }) }
//! ```

#![forbid(unsafe_code, future_incompatible, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, missing_doc_code_examples, unreachable_pub)]

use async_session::chrono::Utc;
use async_session::{Result, Session, SessionStore};
use async_trait::async_trait;
use mongodb::bson;
use mongodb::bson::doc;
use mongodb::options::{ReplaceOptions, SelectionCriteria};
use mongodb::Client;

/// A MongoDB session store.
#[derive(Debug, Clone)]
pub struct MongodbSessionStore {
    client: mongodb::Client,
    db: String,
    coll_name: String,
}

impl MongodbSessionStore {
    /// Create a new instance of `MongodbSessionStore`.
    /// ```rust
    /// # fn main() -> async_session::Result { async_std::task::block_on(async {
    /// # use async_mongodb_session::MongodbSessionStore;
    /// let store =
    /// MongodbSessionStore::connect("mongodb://127.0.0.1:27017", "db_name", "collection")
    /// .await?;
    /// # Ok(()) }) }
    /// ```
    pub async fn connect(uri: &str, db: &str, coll_name: &str) -> mongodb::error::Result<Self> {
        let client = Client::with_uri_str(uri).await?;
        let create_index = doc! {
            "createIndexes": coll_name,
            "indexes": [
                {
                    "key" : {"expireAt": 1 },
                    "name": "session_expire_at_index",
                    "expireAfterSeconds": 0,
                }
            ]
        };
        client
            .database(db)
            .run_command(
                create_index,
                SelectionCriteria::ReadPreference(mongodb::options::ReadPreference::Primary),
            )
            .await?;
        Ok(Self::from_client(client, db, coll_name))
    }

    /// Create a new instance of `MongodbSessionStore` from an open client.
    /// ```rust
    /// use mongodb::{options::ClientOptions, Client};
    ///
    /// # fn main() -> async_session::Result { async_std::task::block_on(async {
    /// # use async_mongodb_session::MongodbSessionStore;
    ///             let client_options = match ClientOptions::parse("mongodb://127.0.0.1:27017").await {
    ///     Ok(c) => c,
    ///     Err(e) => panic!("Client Options Failed: {}", e),
    /// };

    /// let client = match Client::with_options(client_options) {
    ///     Ok(c) => c,
    ///     Err(e) => panic!("Client Creation Failed: {}", e),
    /// };

    /// let store = MongodbSessionStore::from_client(client, "db_name", "collection");
    /// # Ok(()) }) }
    /// ```
    pub fn from_client(client: Client, db: &str, coll_name: &str) -> Self {
        Self {
            client,
            db: db.to_string(),
            coll_name: coll_name.to_string(),
        }
    }
}

#[async_trait]
impl SessionStore for MongodbSessionStore {
    async fn store_session(&self, session: Session) -> Result<Option<String>> {
        let coll = self.client.database(&self.db).collection(&self.coll_name);

        let value = bson::to_bson(&session)?;
        let id = session.id();
        let query = doc! { "session_id": id };
        let replacement = match session.expiry() {
            None => {
                doc! { "session_id": id, "session": value,"created": Utc::now() }
            }
            Some(expire_at) => {
                doc! { "session_id": id, "session": value, "expireAt": expire_at, "created": Utc::now() }
            }
        };

        let opts = ReplaceOptions::builder().upsert(true).build();
        coll.replace_one(query, replacement, Some(opts)).await?;

        Ok(session.into_cookie_value())
    }

    async fn load_session(&self, cookie_value: String) -> Result<Option<Session>> {
        let id = Session::id_from_cookie_value(&cookie_value)?;
        let coll = self.client.database(&self.db).collection(&self.coll_name);
        let filter = doc! { "session_id": id };
        match coll.find_one(filter, None).await? {
            None => Ok(None),
            Some(doc) => {
                let bsession = match doc.get("session") {
                    Some(v) => v.clone(),
                    None => bson::to_bson::<Session>(&Session::new()).unwrap(),
                };
                Ok(Some(bson::from_bson::<Session>(bsession)?))
            }
        }
    }

    async fn destroy_session(&self, session: Session) -> Result {
        let coll = self.client.database(&self.db).collection(&self.coll_name);
        coll.delete_one(doc! { "session_id": session.id() }, None)
            .await?;
        Ok(())
    }

    async fn clear_store(&self) -> Result {
        let coll = self.client.database(&self.db).collection(&self.coll_name);
        coll.drop(None).await?; // does this need to be followed by a create?
        Ok(())
    }
}
