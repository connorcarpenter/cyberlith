//! rustls-acme is an easy-to-use, async compatible ACME client library for rustls.
//! The validation mechanism used is tls-alpn-01, which allows serving acme challenge responses and
//! regular TLS traffic on the same port.
//!
//! rustls-acme is designed to be runtime agnostic and as runtime independent as Rust allows at the
//! moment.
//! No persistent tasks are spawned under the hood and the certificate acquisition/renewal process
//! is folded into the streams and futures being polled by the library user.
//!
//! The goal is to provide a [Let's Encrypt](https://letsencrypt.org/) compatible TLS serving and
//! certificate management using a simple and flexible stream based API.
//!
//! To use rustls-acme add the following lines to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rustls-acme = "*"
//! ```
//!
//! ## High-level API
//!
//! The high-level API consists of a single stream [Incoming] of incoming TLS connection.
//! Polling the next future of the stream takes care of acquisition and renewal of certificates, as
//! well as accepting TLS connections, which are handed over to the caller on success.

//! `examples/high_level.rs` implements a "Hello Tls!" server similar to the one above, which accepts
//! domain, port and cache directory parameters.
//!
//! Note that all examples use the let's encrypt staging directory by default.
//! The production directory imposes strict rate limits, which are easily exhausted accidentally
//! during testing and development.
//! For testing with the staging directory you may open `https://<your domain>:<port>` in a browser
//! that allows TLS connections to servers signed by an untrusted CA (in Firefox click "Advanced..."
//! -> "Accept the Risk and Continue").
//!
//! ## Low-level Rustls API
//!
//! For users who may want to interact with [rustls] or [futures_rustls]
//! directly, the library exposes the underlying certificate management [AcmeState] as well as a
//! matching resolver [ResolvesServerCertAcme] which implements the [rustls::server::ResolvesServerCert] trait.
//! See the server_low_level example on how to use the low-level API directly with [futures_rustls].
//!
//! ## Account and certificate caching
//!
//! A production server using the let's encrypt production directory must implement both account and
//! certificate caching to avoid exhausting the let's encrypt API rate limits.
//! A file based cache using a cache directory is provided by [caches::DirCache].
//! Caches backed by other persistence layers may be implemented using the [Cache] trait,
//! or the underlying [CertCache], [AccountCache] traits (contributions welcome).
//! [caches::CompositeCache] provides a wrapper to combine two implementors of [CertCache] and
//! [AccountCache] into a single [Cache].
//!
//! Note, that the error type parameters of the cache carries over to some other types in this
//! crate via the [AcmeConfig] they are added to.
//! If you want to avoid different specializations based on cache type use the
//! [AcmeConfig::cache_with_boxed_err] method to construct an [AcmeConfig] object.
//!
//!
//! ## The acme module
//!
//! The underlying implementation of an async acme client may be useful to others and is exposed as
//! a module. It is incomplete (contributions welcome) and not covered by any stability
//! promises.
//!
//! ## Special thanks
//!
//! This crate was inspired by the [autocert](https://golang.org/x/crypto/acme/autocert/)
//! package for [Go](https://golang.org).
//!
//! This crate builds on the excellent work of the authors of
//! [rustls](https://github.com/ctz/rustls),
//! [futures-rustls](https://github.com/quininer/futures-rustls),
//! and many others.
//!
//! Thanks to [Josh Triplett](https://github.com/joshtriplett) for contributions and feedback.

mod high_level;
pub use high_level::Config;

mod acceptor;
mod acme;
mod cache;
mod caches;
mod config;
mod helpers;
mod https_helper;
mod incoming;
mod jose;
mod resolver;
mod state;

use ring as crypto;

// TODO: Can we use something like CryptoProvider for rustls, but for ring to drop this requirement?

pub(crate) fn any_ecdsa_type(
    der: &futures_rustls::pki_types::PrivateKeyDer,
) -> Result<
    std::sync::Arc<dyn futures_rustls::rustls::sign::SigningKey>,
    futures_rustls::rustls::Error,
> {
    return futures_rustls::rustls::crypto::ring::sign::any_ecdsa_type(&der);
}

pub(crate) fn crypto_provider() -> futures_rustls::rustls::crypto::CryptoProvider {
    return futures_rustls::rustls::crypto::ring::default_provider();
}
