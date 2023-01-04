#![warn(rust_2018_idioms)]

use bytes::Bytes;
use std::error::Error;
use std::io::{Cursor, Read};

pub use http::{header, Extensions, HeaderMap, Method, Request, Response, StatusCode, Uri};

pub type ConduitRequest = Request<Cursor<Bytes>>;
pub type ResponseResult<Error> = Result<Response<Bytes>, Error>;

pub type BoxError = Box<dyn Error + Send>;
pub type HandlerResult = Result<Response<Bytes>, BoxError>;

/// A helper to convert a concrete error type into a `Box<dyn Error + Send>`
///
/// # Example
///
/// ```
/// # use std::error::Error;
/// # use bytes::Bytes;
/// # use conduit::{box_error, Response};
/// # let _: Result<Response<Bytes>, Box<dyn Error + Send>> =
/// Response::builder().body(Bytes::new()).map_err(box_error);
/// ```
pub fn box_error<E: Error + Send + 'static>(error: E) -> BoxError {
    Box::new(error)
}

pub trait RequestExt {
    /// The request method, such as GET, POST, PUT, DELETE or PATCH
    fn method(&self) -> &Method;

    /// The request URI
    fn uri(&self) -> &Uri;

    /// The byte-size of the body, if any
    fn content_length(&self) -> Option<u64>;

    /// The request's headers, as conduit::Headers.
    fn headers(&self) -> &HeaderMap;

    /// A Reader for the body of the request
    ///
    /// # Blocking
    ///
    /// The returned value implements the blocking `Read` API and should only
    /// be read from while in a blocking context.
    fn body(&mut self) -> &mut dyn Read;

    /// A readable map of extensions
    fn extensions(&self) -> &Extensions;

    /// A mutable map of extensions
    fn extensions_mut(&mut self) -> &mut Extensions;
}

impl RequestExt for ConduitRequest {
    fn method(&self) -> &Method {
        self.method()
    }

    fn uri(&self) -> &Uri {
        self.uri()
    }

    fn content_length(&self) -> Option<u64> {
        Some(self.body().get_ref().len() as u64)
    }

    fn headers(&self) -> &HeaderMap {
        self.headers()
    }

    fn body(&mut self) -> &mut dyn Read {
        self.body_mut()
    }

    fn extensions(&self) -> &Extensions {
        self.extensions()
    }
    fn extensions_mut(&mut self) -> &mut Extensions {
        self.extensions_mut()
    }
}

/// A Handler takes a request and returns a response or an error.
/// By default, a bare function implements `Handler`.
pub trait Handler: Sync + Send + 'static {
    fn call(&self, request: &mut ConduitRequest) -> HandlerResult;
}

impl<F, E> Handler for F
where
    F: Fn(&mut ConduitRequest) -> ResponseResult<E> + Sync + Send + 'static,
    E: Error + Send + 'static,
{
    fn call(&self, request: &mut ConduitRequest) -> HandlerResult {
        (*self)(request).map_err(box_error)
    }
}