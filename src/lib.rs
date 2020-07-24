//! An async-session implementation for MongoDB
//!
//! # Examples
//!
//! ```
//! // tbi
//! ```

#![forbid(unsafe_code, future_incompatible, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, missing_doc_code_examples, unreachable_pub)]

use async_session::{Session, SessionStore};
use async_trait::async_trait;
use mongodb::bson::doc;
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
    pub async fn connect(uri: &str, db: &str, coll_name: &str) -> mongodb::error::Result<Self> {
        let client = Client::with_uri_str(uri).await?;
        Ok(Self::from_client(client, db, coll_name))
    }

    /// Create a new instance of `MongodbSessionStore` from an open client.
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
    async fn store_session(&self, session: Session) -> Option<String> {
        let coll = self.client.database(&self.db).collection(&self.coll_name);

        // TODO: mongodb supports TTL for auto-expiry somehow, need to figure out how!
        let value = serde_json::to_string(&session).ok()?;
        let id = session.id();
        let doc = doc! { "session_id": id, "session": value };
        coll.insert_one(doc, None).await.ok()?;

        session.into_cookie_value()
    }

    async fn load_session(&self, cookie_value: String) -> Option<Session> {
        let id = Session::id_from_cookie_value(&cookie_value).ok()?;
        let coll = self.client.database(&self.db).collection(&self.coll_name);

        let filter = doc! { "session_id": id };
        let doc = coll.find_one(filter, None).await.ok()??;
        let value = doc.get_str("session").ok()?;
        serde_json::from_str(value).ok()
    }

    async fn destroy_session(&self, _session: Session) -> async_session::Result {
        todo!();
    }

    async fn clear_store(&self) -> async_session::Result {
        todo!();
    }
}
