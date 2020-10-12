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

## Configuration

A `created` property is available on the root of the session document that so the [expiry feature](https://docs.mongodb.com/manual/tutorial/expire-data/#expire-documents-after-a-specified-number-of-seconds) can be used in the configuration.

If your application code to create a session store is something like:
```
let store = MongodbSessionStore::connect("mongodb://127.0.0.1:27017", "db_name", "coll_session").await?;
```

Then the script to create the expiry would be:
```
use db_name;
db.coll_session.createIndex( { "created": 1 } , { expireAfterSeconds: 300 } );
```

If you wish to redefine the session duration then the index must be dropped first using:
```
use db_name;
db.coll_session.dropIndex( { "created": 1 })
db.coll_session.createIndex( { "created": 1 } , { expireAfterSeconds: 300 } );
```

Other way to set create the index is using  `create_created_index_for_global_expiry` passing the amount of seconds to expiry after the session.

Also, an `expireAt` property is available on the root of the session document IFF the session expire is set. Note that  [async-session doesn't set by default](https://github.com/http-rs/async-session/blob/main/src/session.rs#L98).

To enable this [expiry feature](https://docs.mongodb.com/manual/tutorial/expire-data/#expire-documents-at-a-specific-clock-time) at `index` for `expireAt` should be created calling `create_expire_at_index` function or with this script ( following the above example )

```
use db_name;
db.coll_session.createIndex( { "expireAt": 1 } , { expireAfterSeconds: 0 } );
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
