//! If you want to render templates, see the [`Render`](trait.Render.html)
//! trait.
//!
//! To customise the templating behavior, see the
//! [`TemplateSupport`](trait.TemplateSupport.html) trait.

#![deny(missing_docs)]

#[macro_use]
extern crate nickel;
extern crate mustache;
extern crate rustc_serialize;

use rustc_serialize::Encodable;
use mustache::{Data, Template};

use std::borrow::Cow;
use std::path::Path;

mod default_implementations;
mod response_extension;

/// Extension trait for common `mustache::Template` usage.
pub trait Render {
    /// Return type for all of the extension methods.
    type Output;

    /// Renders a `mustache::Template` with specific `Encodable` data
    ///
    /// See `examples/example.rs` for example usage.
    fn render<T, P>(self, path: P, data: &T) -> Self::Output
    where T: Encodable,
          P: AsRef<Path>;

    /// Renders a `mustache::Template` wrapped inside a specific layout
    ///
    /// See `examples/with_layout.rs` for example usage.
    fn render_with_layout<T, P, L>(self, path: P, layout: L, data: &T) -> Self::Output
    where T: Encodable,
          P: AsRef<Path>,
          L: AsRef<Path>;

    /// Renders a `mustache::Template` with specific `mustache::Data`
    ///
    /// See `examples/helper_functions.rs` for example usage.
    fn render_data<P>(self, path: P, data: &Data) -> Self::Output where P: AsRef<Path>;

    /// Renders a `mustache::Template` wrapped inside a specific layout
    ///
    /// See `examples/with_layout.rs` for example usage.
    fn render_data_with_layout<P, L>(self, path: P, layout: L, data: &Data) -> Self::Output
    where P: AsRef<Path>,
          L: AsRef<Path>;
}

/// Customise the behaviour of the templating system.
pub trait TemplateSupport {
    /// What type to dispatch the cache handling to.
    ///
    /// # Note
    ///
    /// Currently if you don't want custom behavior you should use `()` as your
    /// `Cache`. When 'associated type defaults' becomes stable then this won't
    /// be necessary to specify anymore.
    type Cache: TemplateCache;

    /// A reference to the `Cache` if there is one.
    fn cache(&self) -> Option<&Self::Cache> {
        None
    }

    /// Adjust the path of a template lookup before it gets compiled.
    ///
    /// This can be useful if you want to keep a clean directory structure
    /// without having to spread that knowledge across your handlers.
    ///
    /// See `examples/adjusted_path.rs` for example usage.
    fn adjust_path<'a>(&self, path: &'a Path) -> Cow<'a, Path> {
        Cow::Borrowed(path)
    }

    /// Adjust the path of a layout lookup before it gets compiled.
    ///
    /// This can be useful if you want to keep a clean directory structure
    /// without having to spread that knowledge across your handlers.
    ///
    /// See `examples/adjusted_path.rs` for example usage.
    fn adjust_layout_path<'a>(&self, path: &'a Path) -> Cow<'a, Path> {
        Cow::Borrowed(path)
    }

    /// The default layout to use when rendering.
    ///
    /// See `examples/default_layout.rs` for example usage.
    fn default_layout(&self) -> Option<Cow<Path>> {
        None
    }
}

/// Handle template caching through a borrowed reference.
pub trait TemplateCache {
    /// Handles a cache lookup for a given template.
    ///
    /// # Expected behavior
    /// ```not_rust
    /// if let Some(template) = cache.get(path) {
    ///     return handle(template)
    /// } else {
    ///     let template = on_miss(path);
    ///     return handle(template)
    /// }
    /// ```
    ///
    /// # Fix-me!
    /// The signature is a bit crazy, but due to the nature of the interior mutability
    /// required it's difficult to express a general interface without restrictions
    /// on the kinds of type `TemplateCache` could be implemented for.
    ///
    /// Any improvements to get it to a more `entry`-like design are welcome!
    fn handle<'a, P, F, R>(&self, path: &'a Path, handle: P, on_miss: F) -> R
    where P: FnOnce(Result<&Template, CompileError>) -> R,
          F: FnOnce(&'a Path) -> Result<Template, CompileError>;
}

/// Currently the errors are `String`s
pub type CompileError = String;
