//! `Retry-After` as accessible `impl headers::Header`
//!
//! The [`headers` crate](https://crates.io/crates/headers) contains a [`Retry-After`](https://docs.rs/headers::%22RetryAfter%22)
//! [header](https://docs.rs/headers::%22Header%22) [RFC 7231](https://datatracker.ietf.org/doc/html/rfc7231#section-7.1.3) implementation,
//! but at the time of creating this crate here, v0.4.1 `headers::RetryAfter` is not accessible,
//! i.e. you can not read it as a delay or date value, only write.
//!
//! This crate here contains its own, accessible [`Retry-After` RFC 7231 header implementation](RetryAfter),
//! using the same [`httpdate` crate](https://crates.io/crates/httpdate) as `headers::RetryAfter`.
#![deny(unsafe_code)]

use headers_core::{Header, HeaderName, HeaderValue};
use std::convert::{TryFrom, TryInto};
use std::time::{Duration, SystemTime};

/// The `Retry-After` header.
///
/// The `Retry-After` response-header field can be used with a 503 (Service
/// Unavailable) response to indicate how long the service is expected to be
/// unavailable to the requesting client. This field MAY also be used with any
/// 3xx (Redirection) response to indicate the minimum time the user-agent is
/// asked wait before issuing the redirected request. The value of this field
/// can be either an HTTP-date or an integer number of seconds (in decimal)
/// after the time of the response.
///
/// # Examples
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use headers::{HeaderMap, HeaderMapExt as _};
/// use headers_retry_after::RetryAfter;
/// use http::header::RETRY_AFTER;
/// use std::time::{Duration, SystemTime};
///
/// let delay = RetryAfter::delay(Duration::from_secs(300));
/// let date = RetryAfter::date(SystemTime::UNIX_EPOCH);
///
/// let mut headers = HeaderMap::new();
///
/// headers.insert(RETRY_AFTER, "300".parse()?);
/// assert_eq!(Some(delay), headers.typed_get());
///
/// headers.insert(RETRY_AFTER, "Thu, 01 Jan 1970 00:00:00 GMT".parse()?);
/// assert_eq!(Some(date), headers.typed_get());
/// # Ok(())
/// # }
/// ```
///
/// Retry-After header, defined in [RFC7231](https://datatracker.ietf.org/doc/html/rfc7231#section-7.1.3)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetryAfter {
    /// Retry after the given date
    Date(SystemTime),
    /// Retry after the given delay
    Delay(Duration),
}

impl RetryAfter {
    /// Create an `RetryAfter` header with a date value.
    pub fn date(time: SystemTime) -> RetryAfter {
        RetryAfter::Date(time)
    }

    /// Create an `RetryAfter` header with a delay value.
    pub fn delay(dur: Duration) -> RetryAfter {
        RetryAfter::Delay(dur)
    }
}

/// Error for `RetryAfter::try_from(header_value)`
#[derive(Debug)]
pub struct ParseHeaderValueError(());

impl std::fmt::Display for ParseHeaderValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "can not parse parse header value")
    }
}

impl std::error::Error for ParseHeaderValueError {}

impl TryFrom<&HeaderValue> for RetryAfter {
    type Error = ParseHeaderValueError;

    fn try_from(value: &HeaderValue) -> Result<Self, Self::Error> {
        let value =
            std::str::from_utf8(value.as_bytes()).map_err(|_e| ParseHeaderValueError(()))?;

        if let Ok(seconds) = value.parse::<u64>() {
            let dur = Duration::from_secs(seconds);
            return Ok(Self::delay(dur));
        }

        let time = httpdate::parse_http_date(value).map_err(|_e| ParseHeaderValueError(()))?;
        Ok(Self::date(time))
    }
}

impl TryFrom<HeaderValue> for RetryAfter {
    type Error = ParseHeaderValueError;

    fn try_from(value: HeaderValue) -> Result<Self, Self::Error> {
        TryFrom::try_from(&value)
    }
}

impl From<&RetryAfter> for HeaderValue {
    fn from(value: &RetryAfter) -> Self {
        match value {
            RetryAfter::Date(time) => {
                let s = httpdate::fmt_http_date(*time);
                HeaderValue::from_maybe_shared(s).expect("HTTP date always is a valid value")
            }
            RetryAfter::Delay(ref dur) => dur.as_secs().into(),
        }
    }
}

impl From<RetryAfter> for HeaderValue {
    fn from(value: RetryAfter) -> Self {
        From::from(&value)
    }
}

impl Header for RetryAfter {
    fn name() -> &'static HeaderName {
        &headers_core::header::RETRY_AFTER
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers_core::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        if let Some(value) = values.next() {
            return value
                .try_into()
                .map_err(|_e| headers_core::Error::invalid());
        }
        Err(headers_core::Error::invalid())
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        values.extend(std::iter::once(self.into()));
    }
}

impl From<SystemTime> for RetryAfter {
    fn from(value: SystemTime) -> Self {
        Self::date(value)
    }
}

impl From<Duration> for RetryAfter {
    fn from(value: Duration) -> Self {
        Self::delay(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_decode<T: headers::Header>(values: &[&str]) -> Option<T> {
        use headers::HeaderMapExt;
        let mut map = http::header::HeaderMap::new();
        for val in values {
            map.append(T::name(), val.parse().unwrap());
        }
        map.typed_get()
    }

    #[test]
    fn delay_decode() {
        let r: RetryAfter = test_decode(&["1234"]).unwrap();
        assert_eq!(r, RetryAfter::delay(Duration::from_secs(1234)),);
    }

    macro_rules! test_retry_after_datetime {
        ($name:ident, $s:expr) => {
            #[test]
            fn $name() {
                let r: RetryAfter = test_decode(&[$s]).unwrap();
                let dt = httpdate::parse_http_date("Sun, 06 Nov 1994 08:49:37 GMT").unwrap();

                assert_eq!(r, RetryAfter::date(dt));
            }
        };
    }

    test_retry_after_datetime!(date_decode_rfc1123, "Sun, 06 Nov 1994 08:49:37 GMT");
    test_retry_after_datetime!(date_decode_rfc850, "Sunday, 06-Nov-94 08:49:37 GMT");
    test_retry_after_datetime!(date_decode_asctime, "Sun Nov  6 08:49:37 1994");
}

#[cfg(doctest)]
#[doc=include_str!("../README-crate.md")]
mod readme {}
