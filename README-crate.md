Typed accessible `Retry-After` HTTP `headers::Header` impl

# Usage

```rust
use headers::{HeaderMap, HeaderMapExt as _};
use headers_retry_after::RetryAfter;
use http::header::RETRY_AFTER;
use std::time::{Duration, SystemTime};

fn example() -> Result<(), Box<dyn std::error::Error>> {
    let delay = RetryAfter::delay(Duration::from_secs(300));
    let date = RetryAfter::date(SystemTime::UNIX_EPOCH);

    let mut headers = HeaderMap::new();

    headers.insert(RETRY_AFTER, "300".parse()?);
    assert_eq!(Some(delay), headers.typed_get());

    headers.insert(RETRY_AFTER, "Thu, 01 Jan 1970 00:00:00 GMT".parse()?);
    assert_eq!(Some(date), headers.typed_get());

    Ok(())
}
```

# MSRV

Same as `headers@0.4.1`, i.e. Rust 1.56

# License

Same as `headers@0.4.1`, i.e. MIT only
