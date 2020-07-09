//! Middleware types
//!
//! # Examples
//! ```no_run
//! use futures::future::BoxFuture;
//! use surf::middleware::{Next, Middleware};
//! use surf::{Client, Request, Response, Result};
//! use std::time;
//! use std::sync::Arc;
//!
//! /// Log each request's duration
//! #[derive(Debug)]
//! pub struct Logger;
//!
//! impl Middleware for Logger {
//!     fn handle<'a>(
//!         &'a self,
//!         req: Request,
//!         client: Client,
//!         next: Next<'a>,
//!     ) -> BoxFuture<'a, Result<Response>> {
//!         Box::pin(async move {
//!             println!("sending request to {}", req.url());
//!             let now = time::Instant::now();
//!             let res = next.run(req, client).await?;
//!             println!("request completed ({:?})", now.elapsed());
//!             Ok(res)
//!         })
//!     }
//! }
//! ```
//! `Middleware` can also be instantiated using a free function thanks to some convenient trait
//! implementations.
//!
//! ```no_run
//! use futures::future::BoxFuture;
//! use surf::middleware::{Next, Middleware};
//! use surf::{Client, Request, Response, Result};
//! use std::time;
//! use std::sync::Arc;
//!
//! fn logger<'a>(req: Request, client: Client, next: Next<'a>) -> BoxFuture<'a, Result<Response>> {
//!     Box::pin(async move {
//!         println!("sending request to {}", req.url());
//!         let now = time::Instant::now();
//!         let res = next.run(req, client).await?;
//!         println!("request completed ({:?})", now.elapsed());
//!         Ok(res)
//!     })
//! }
//! ```

use std::sync::Arc;

use crate::{Client, Request, Response, Result};

pub mod logger;
mod redirect;

pub use redirect::Redirect;

use futures::future::BoxFuture;

/// Middleware that wraps around remaining middleware chain.
pub trait Middleware: 'static + Send + Sync {
    /// Asynchronously handle the request, and return a response.
    fn handle<'a>(
        &'a self,
        req: Request,
        client: Client,
        next: Next<'a>,
    ) -> BoxFuture<'a, Result<Response>>;
}

// This allows functions to work as middleware too.
impl<F> Middleware for F
where
    F: Send
        + Sync
        + 'static
        + for<'a> Fn(Request, Client, Next<'a>) -> BoxFuture<'a, Result<Response>>,
{
    fn handle<'a>(
        &'a self,
        req: Request,
        client: Client,
        next: Next<'a>,
    ) -> BoxFuture<'a, Result<Response>> {
        (self)(req, client, next)
    }
}

/// The remainder of a middleware chain, including the endpoint.
#[allow(missing_debug_implementations)]
pub struct Next<'a> {
    next_middleware: &'a [Arc<dyn Middleware>],
    endpoint: &'a (dyn (Fn(Request, Client) -> BoxFuture<'static, Result<Response>>)
             + Send
             + Sync
             + 'static),
}

impl Clone for Next<'_> {
    fn clone(&self) -> Self {
        Self {
            next_middleware: self.next_middleware,
            endpoint: self.endpoint,
        }
    }
}

impl Copy for Next<'_> {}

impl<'a> Next<'a> {
    /// Create a new instance
    pub fn new(
        next: &'a [Arc<dyn Middleware>],
        endpoint: &'a (dyn (Fn(Request, Client) -> BoxFuture<'static, Result<Response>>)
                 + Send
                 + Sync
                 + 'static),
    ) -> Self {
        Self {
            endpoint,
            next_middleware: next,
        }
    }

    /// Asynchronously execute the remaining middleware chain.
    pub fn run(mut self, req: Request, client: Client) -> BoxFuture<'a, Result<Response>> {
        if let Some((current, next)) = self.next_middleware.split_first() {
            self.next_middleware = next;
            current.handle(req, client, self)
        } else {
            (self.endpoint)(req, client)
        }
    }
}
