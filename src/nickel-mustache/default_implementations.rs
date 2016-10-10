use {TemplateSupport, TemplateCache, CompileError};

use mustache::Template;

use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::collections::HashMap;
use std::collections::hash_map::Entry::*;

impl TemplateSupport for () {
    type Cache = ();
}

impl TemplateCache for () {
    fn handle<'a, P, F, R>(&self, _: &'a Path, _: P, _: F) -> R
    where P: FnOnce(Result<&Template, CompileError>) -> R,
          F: FnOnce(&'a Path) -> Result<Template, CompileError> {
        unreachable!()
    }
}

impl TemplateCache for RwLock<HashMap<PathBuf, Template>> {
    fn handle<'a, P, F, R>(&self, path: &'a Path, handle: P, on_miss: F) -> R
    where P: FnOnce(Result<&Template, CompileError>) -> R,
          F: FnOnce(&'a Path) -> Result<Template, CompileError> {
        // Fast path doesn't need writer lock
        if let Some(t) = self.read().unwrap().get(path) {
            return handle(Ok(t));
        }

        // We didn't find the template, get writers lock
        let mut templates = self.write().unwrap();

        // Search again incase there was a race to compile the template
        let template = match templates.entry(path.into()) {
            Vacant(entry) => {
                let template = match on_miss(path) {
                    Ok(template) => template,
                    Err(e) => return handle(Err(e)),
                };
                entry.insert(template)
            }
            Occupied(entry) => entry.into_mut(),
        };

        handle(Ok(template))
    }
}
