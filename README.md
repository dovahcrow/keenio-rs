Keen IO API v3 binding for rust [![Build Status](https://travis-ci.org/doomsplayer/keenio-rs.png?branch=master)](https://travis-ci.org/doomsplayer/keenio-rs)
======================================

Keen IO is a set of APIs for streaming, computing and visualizing data from Internet. This repository provides a Rust binding for Keen IO APIs.

To call for Keen IO API, you first need to create a `KeenClient` instance:
```rust
let key = env::var("KEEN_READ_KEY").unwrap();
let proj = env::var("KEEN_PROJECT_ID").unwrap();
let mut client = KeenClient::new(&key, &proj);
```

Then you can get a `KeenQuery` instance from the `KeenClient` instance:
```rust
let mut q = client.query(metric.clone(),
                         "strikingly_pageviews".into(),
                         TimeFrame::Absolute(UTC::now() - Duration::hours(10),
                                             UTC::now() - Duration::hours(1)));
```

Now you can run query with `KeenQuery` instance:
```rust
q.filter(Filter::lt("pageId", args.to));
q.group_by("normalized_referrer");
```

Currently this binding supports `group_by`, `filter`, `interval`, `max_age`. Other queries can also be added via `other` method, providing key and value. `url` method in `KeenQuery` instance gives the keen query url that you may need.

See full example at `examples/main.rs`



[documents](http://doomsplayer.github.io/keenio-rs/keen/index.html)
