#[cfg(test)]
mod tests {
    use async_mongodb_session::*;
    use async_session::{Session, SessionStore};
    use lazy_static::lazy_static;
    use mongodb::{options::ClientOptions, Client};
    use rand::Rng;
    use std::env;

    lazy_static! {
        static ref HOST: String = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        static ref PORT: String = env::var("PORT").unwrap_or_else(|_| "27017".to_string());
        static ref DATABASE: String = env::var("DATABASE").unwrap_or_else(|_| "db_name".to_string());
        static ref COLLECTION: String = env::var("COLLECTION").unwrap_or_else(|_| "collection".to_string());
        static ref CONNECTION_STRING: String =
            format!("mongodb://{}:{}/", HOST.as_str(), PORT.as_str());
    }

    #[test]
    fn test_from_client() -> async_session::Result {
        async_std::task::block_on(async {
            let client_options = match ClientOptions::parse(&*CONNECTION_STRING).await {
                Ok(c) => c,
                Err(e) => panic!("Client Options Failed: {}", e),
            };

            let client = match Client::with_options(client_options) {
                Ok(c) => c,
                Err(e) => panic!("Client Creation Failed: {}", e),
            };
            
            let store = MongodbSessionStore::from_client(client, &DATABASE, &COLLECTION);
            let mut rng = rand::thread_rng();
            let n2: u16 = rng.gen();
            let key = format!("key-{}", n2);
            let value = format!("value-{}", n2);
            let mut session = Session::new();
            session.insert(&key, &value)?;

            let cookie_value = store.store_session(session).await?.unwrap();
            let session = store.load_session(cookie_value).await?.unwrap();
            assert_eq!(&session.get::<String>(&key).unwrap(), &value);

            Ok(())
        })
    }

    #[test]
    fn test_new() -> async_session::Result {
        async_std::task::block_on(async {
            let store =
                MongodbSessionStore::new(&CONNECTION_STRING, &DATABASE, &COLLECTION).await?;

            let mut rng = rand::thread_rng();
            let n2: u16 = rng.gen();
            let key = format!("key-{}", n2);
            let value = format!("value-{}", n2);
            let mut session = Session::new();
            session.insert(&key, &value)?;

            let cookie_value = store.store_session(session).await?.unwrap();
            let session = store.load_session(cookie_value).await?.unwrap();
            assert_eq!(&session.get::<String>(&key).unwrap(), &value);

            Ok(())
        })
    }

    #[test]
    fn test_with_expire() -> async_session::Result {
        async_std::task::block_on(async {
            let store =
                MongodbSessionStore::new(&CONNECTION_STRING, &DATABASE, &COLLECTION).await?;

            store.initialize().await?;

            let mut rng = rand::thread_rng();
            let n2: u16 = rng.gen();
            let key = format!("key-{}", n2);
            let value = format!("value-{}", n2);
            let mut session = Session::new();
            session.expire_in(std::time::Duration::from_secs(5));
            session.insert(&key, &value)?;

            let cookie_value = store.store_session(session).await?.unwrap();
            let session = store.load_session(cookie_value).await?.unwrap();
            assert_eq!(&session.get::<String>(&key).unwrap(), &value);

            Ok(())
        })
    }

    #[test]
    fn test_check_expired() -> async_session::Result {
        use async_std::task;
        use std::time::Duration;
        async_std::task::block_on(async {
            let store =
                MongodbSessionStore::new(&CONNECTION_STRING, &DATABASE, &COLLECTION).await?;

            store.initialize().await?;

            let mut rng = rand::thread_rng();
            let n2: u16 = rng.gen();
            let key = format!("key-{}", n2);
            let value = format!("value-{}", n2);
            let mut session = Session::new();
            session.expire_in(Duration::from_secs(1));
            session.insert(&key, &value)?;

            let cookie_value = store.store_session(session).await?.unwrap();

            task::sleep(Duration::from_secs(1)).await;
            let session_to_recover = store.load_session(cookie_value).await?;

            assert!(&session_to_recover.is_none());

            Ok(())
        })
    }
}
