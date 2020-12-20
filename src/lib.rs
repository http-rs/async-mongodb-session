//! An async-session implementation for MongoDB
//!
//! # Examples
//!
//! ```
//! use async_mongodb_session::*;
//! use async_session::{Session, SessionStore};
//!
//! # fn main() -> async_session::Result { async_std::task::block_on(async {
//! let store = MongodbSessionStore::new("mongodb://127.0.0.1:27017", "db_name", "collection");
//! # Ok(()) }) }
//! ```

#![forbid(unsafe_code, future_incompatible, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, missing_doc_code_examples, unreachable_pub)]

use async_session::chrono::{Duration, Utc};
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
    ttl: usize,
}

impl MongodbSessionStore {
    /// Create a new instance of `MongodbSessionStore` after stablish the connection to monngodb.
    /// ```rust
    /// # fn main() -> async_session::Result { async_std::task::block_on(async {
    /// # use async_mongodb_session::MongodbSessionStore;
    /// let store =
    /// MongodbSessionStore::new("mongodb://127.0.0.1:27017", "db_name", "collection")
    /// .await?;
    /// # Ok(()) }) }
    /// ```
    pub async fn new(uri: &str, db: &str, coll_name: &str) -> mongodb::error::Result<Self> {
        let client = Client::with_uri_str(uri).await?;
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
            ttl: 1200, // 20 mins by default.
        }
    }

    /// Initialize the default expiration mechanism, based on the document expiration
    /// that mongodb provides https://docs.mongodb.com/manual/tutorial/expire-data/#expire-documents-at-a-specific-clock-time.
    /// The default ttl applyed to sessions without expiry is 20 minutes.
    /// If the `expireAt` date field contains a date in the past, mongodb considers the document expired and will be deleted.
    /// Note: mongodb runs the expiration logic every 60 seconds.
    /// ```rust
    /// # fn main() -> async_session::Result { async_std::task::block_on(async {
    /// # use async_mongodb_session::MongodbSessionStore;
    /// let store =
    /// MongodbSessionStore::new("mongodb://127.0.0.1:27017", "db_name", "collection")
    /// .await?;
    /// store.initialize().await?;
    /// # Ok(()) }) }
    /// ```
    pub async fn initialize(&self) -> Result {
        &self.index_on_expiry_at().await?;
        Ok(())
    }

    /// Get the default ttl value in seconds.
    /// ```rust
    /// # fn main() -> async_session::Result { async_std::task::block_on(async {
    /// # use async_mongodb_session::MongodbSessionStore;
    /// let store =
    /// MongodbSessionStore::new("mongodb://127.0.0.1:27017", "db_name", "collection")
    /// .await?;
    /// let ttl = store.ttl();
    /// # Ok(()) }) }
    /// ```
    pub fn ttl(&self) -> usize {
        self.ttl
    }

    /// Set the default ttl value in seconds.
    /// ```rust
    /// # fn main() -> async_session::Result { async_std::task::block_on(async {
    /// # use async_mongodb_session::MongodbSessionStore;
    /// let mut store =
    /// MongodbSessionStore::new("mongodb://127.0.0.1:27017", "db_name", "collection")
    /// .await?;
    /// store.set_ttl(300);
    /// # Ok(()) }) }
    /// ```
    pub fn set_ttl(&mut self, ttl: usize) {
        self.ttl = ttl;
    }

    /// private associated function
    /// Create an `expire after seconds` index in the provided field.
    /// Testing is covered by initialize test.
    async fn create_expire_index(&self, field_name: &str, expire_after_seconds: u32) -> Result {
        let create_index = doc! {
            "createIndexes": &self.coll_name,
            "indexes": [
                {
                    "key" : { field_name: 1 },
                    "name": format!("session_expire_index_{}", field_name),
                    "expireAfterSeconds": expire_after_seconds,
                }
            ]
        };
        self.client
            .database(&self.db)
            .run_command(
                create_index,
                SelectionCriteria::ReadPreference(mongodb::options::ReadPreference::Primary),
            )
            .await?;
        Ok(())
    }

    /// Create a new index for the `created` property and set the expiry ttl (in secods).
    /// The session will expire when the number of seconds in the expireAfterSeconds field has passed
    /// since the time specified in its created field.
    /// https://docs.mongodb.com/manual/tutorial/expire-data/#expire-documents-after-a-specified-number-of-seconds
    /// ```rust
    /// # fn main() -> async_session::Result { async_std::task::block_on(async {
    /// # use async_mongodb_session::MongodbSessionStore;
    /// let store =
    /// MongodbSessionStore::new("mongodb://127.0.0.1:27017", "db_name", "collection")
    /// .await?;
    /// store.index_on_created(300).await?;
    /// # Ok(()) }) }
    /// ```
    pub async fn index_on_created(&self, expire_after_seconds: u32) -> Result {
        self.create_expire_index("created", expire_after_seconds)
            .await?;
        Ok(())
    }

    /// Create a new index for the `expireAt` property, allowing to expire sessions at a specific clock time.
    /// If the `expireAt` date field contains a date in the past, mongodb considers the document expired and will be deleted.
    /// https://docs.mongodb.com/manual/tutorial/expire-data/#expire-documents-at-a-specific-clock-time
    /// ```rust
    /// # fn main() -> async_session::Result { async_std::task::block_on(async {
    /// # use async_mongodb_session::MongodbSessionStore;
    /// let store =
    /// MongodbSessionStore::new("mongodb://127.0.0.1:27017", "db_name", "collection")
    /// .await?;
    /// store.index_on_expiry_at().await?;
    /// # Ok(()) }) }
    /// ```
    pub async fn index_on_expiry_at(&self) -> Result {
        self.create_expire_index("expireAt", 0).await?;
        Ok(())
    }
}

#[async_trait]
impl SessionStore for MongodbSessionStore {
    async fn store_session(&self, session: Session) -> Result<Option<String>> {
        let coll = self.client.database(&self.db).collection(&self.coll_name);

        let value = bson::to_bson(&session)?;
        let id = session.id();
        let query = doc! { "session_id": id };
        let expire_at = match session.expiry() {
            None => Utc::now() + Duration::from_std(std::time::Duration::from_secs(1200)).unwrap(),

            Some(expiry) => *{ expiry },
        };
        let replacement = doc! { "session_id": id, "session": value, "expireAt": expire_at, "created": Utc::now() };

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
        coll.drop(None).await?;
        self.initialize().await?;
        Ok(())
    }
}
