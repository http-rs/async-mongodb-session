<h1 align="center">async-mongodb-session</h1>
<div align="center">
  <strong>
    An async-session implementation for MongoDB
  </strong>
</div>

<br />

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/async-mongodb-session">
    <img src="https://img.shields.io/crates/v/async-mongodb-session.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/async-mongodb-session">
    <img src="https://img.shields.io/crates/d/async-mongodb-session.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/async-mongodb-session">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
</div>

<div align="center">
  <h3>
    <a href="https://docs.rs/async-mongodb-session">
      API Docs
    </a>
    <span> | </span>
    <a href="https://github.com/yoshuawuyts/async-mongodb-session/releases">
      Releases
    </a>
    <span> | </span>
    <a href="https://github.com/yoshuawuyts/async-mongodb-session/blob/master.github/CONTRIBUTING.md">
      Contributing
    </a>
  </h3>
</div>

## Installation
```sh
$ cargo add async-mongodb-session
```

## Overview
This library utilises the document expiration feature in mongodb and is based on [specific clock time](https://docs.mongodb.com/manual/tutorial/expire-data/#expire-documents-at-a-specific-clock-time).

The expiry index is applied to the collection when a new session object is created. 

As document expiration is a [background task](https://docs.mongodb.com/manual/core/index-ttl/#timing-of-the-delete-operation) some stale sessions may be present in the database but won't be returned by this library.

## Example with tide
Create an HTTP server that keep track of user visits in the session.

```rust
#[async_std::main]
async fn main() -> tide::Result<()> {
    tide::log::start();
    let mut app = tide::new();

    app.with(tide::sessions::SessionMiddleware::new(
        MongodbSessionStore::new("mongodb://127.0.0.1:27017", "db_name", "collection").await?,
        std::env::var("TIDE_SECRET")
            .expect(
                "Please provide a TIDE_SECRET value of at \
                      least 32 bytes in order to run this example",
            )
            .as_bytes(),
    ));

    app.with(tide::utils::Before(
        |mut request: tide::Request<()>| async move {
            let session = request.session_mut();
            let visits: usize = session.get("visits").unwrap_or_default();
            session.insert("visits", visits + 1).unwrap();
            request
        },
    ));

    app.at("/").get(|req: tide::Request<()>| async move {
        let visits: usize = req.session().get("visits").unwrap();
        Ok(format!("you have visited this website {} times", visits))
    });

    app.at("/reset")
        .get(|mut req: tide::Request<()>| async move {
            req.session_mut().destroy();
            Ok(tide::Redirect::new("/"))
        });

    app.listen("127.0.0.1:8080").await?;

    Ok(())
}
```

## Test

The tests rely on an running instance of mongodb either on your local machine or remote.
The quickest way to get an instance up and running locally is using the following docker command:

```
$ docker run -d -p 27017:27017 -v ~/data:/data/db mongo:4.2
```

The tests can then be executed with
```
$ cargo test
```

The default settings for the mongodb instance is set to 127.0.0.1:27017 but that can be over ridden by setting the HOST and PORT environment variables.
```
$ HOST=mymongo.com PORT=1234 cargo test
```

## Safety
This crate uses ``#![deny(unsafe_code)]`` to ensure everything is implemented in
100% Safe Rust.

## Contributing
Want to join us? Check out our ["Contributing" guide][contributing] and take a
look at some of these issues:

- [Issues labeled "good first issue"][good-first-issue]
- [Issues labeled "help wanted"][help-wanted]

[contributing]: https://github.com/yoshuawuyts/async-mongodb-session/blob/master.github/CONTRIBUTING.md
[good-first-issue]: https://github.com/yoshuawuyts/async-mongodb-session/labels/good%20first%20issue
[help-wanted]: https://github.com/yoshuawuyts/async-mongodb-session/labels/help%20wanted

## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br/>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
