use {TemplateSupport, TemplateCache, Render, CompileError};

use rustc_serialize::Encodable;
use mustache::{self, Data, Template};
use nickel::{Response, MiddlewareResult, Halt};

use std::io::Write;
use std::path::Path;
use std::fmt::Debug;

impl<'a, 'mw, D> Render for Response<'mw, D>
where D: TemplateSupport {
    type Output = MiddlewareResult<'mw, D>;

    fn render<T, P>(self, path: P, data: &T) -> Self::Output
    where T: Encodable,
          P: AsRef<Path> {
        render(self, path.as_ref(), |template, mut stream| {
            template.render(&mut stream, data)
        })
    }

    fn render_with_layout<T, P, L>(self, path: P, layout: L, data: &T) -> Self::Output
    where T: Encodable,
          P: AsRef<Path>,
          L: AsRef<Path> {
        render_with_layout(self, path.as_ref(), Some(layout.as_ref()), |template, mut stream| {
            template.render(&mut stream, data)
        })
    }

    fn render_data<P>(self, path: P, data: &Data) -> Self::Output
    where P: AsRef<Path> {
        render(self, path.as_ref(), |template, mut stream| {
            template.render_data(&mut stream, data);
            Ok(())
        })
    }

    fn render_data_with_layout<P, L>(self, path: P, layout: L, data: &Data) -> Self::Output
    where P: AsRef<Path>,
          L: AsRef<Path> {
        render_with_layout(self, path.as_ref(), Some(layout.as_ref()), |template, mut stream| {
            template.render_data(&mut stream, data);
            Ok(())
        })
    }
}

trait RenderSteps<'a> : Sized {
    type Result;
    type ServerData: TemplateSupport + 'a;

    fn data(&self) -> &'a Self::ServerData;

    fn error(self, CompileError) -> Self::Result;

    fn write<F, E>(self, F) -> Self::Result
    where F: FnOnce(&mut Write) -> Result<(), E>,
          E: Debug;
}

impl<'mw, D> RenderSteps<'mw> for Response<'mw, D>
where D: TemplateSupport + 'mw {
    type Result = MiddlewareResult<'mw, D>;
    type ServerData = D;

    fn data(&self) -> &'mw Self::ServerData {
        self.server_data()
    }

    fn error(self, e: CompileError) -> Self::Result {
        try_with!(self, Err(e))
    }

    fn write<F, E>(self, f: F) -> Self::Result
    where F: FnOnce(&mut Write) -> Result<(), E>,
          E: Debug {
        let mut stream = try!(self.start());

        match f(&mut stream) {
            Ok(()) => Ok(Halt(stream)),
            Err(e) => stream.bail(format!("Problem rendering template: {:?}", e)),
        }
    }
}

fn render<'a, T, F>(t: T, path: &Path, f: F) -> T::Result
where T: RenderSteps<'a>,
      F: FnOnce(&Template, &mut Write) -> Result<(), mustache::Error> {
    let default_layout = t.data().default_layout();
    render_with_layout(t, path, default_layout, f)
}

fn render_with_layout<'a, T, L, F>(t: T, inner: &Path, layout: Option<L>, f: F) -> T::Result
where T: RenderSteps<'a>,
      L: AsRef<Path>,
      F: FnOnce(&Template, &mut Write) -> Result<(), mustache::Error> {
    let data = t.data();

    let inner = &*data.adjust_path(inner);

    macro_rules! try_t {
        ($e:expr, $msg:expr) => (
            try_t!($e.map_err(|e| format!("{}: {:?}", $msg, e)))
        );
        ($e:expr) => (
            match $e {
                Ok(template) => template,
                Err(e) => return t.error(e)
            }
        )
    }

    if let Some(layout) = layout {
        // Render a layout!
        cached_compile(inner, data, |template| {
            let template = try_t!(template);

            // render inner template to buffer
            let mut buf = vec![];
            try_t!(f(template, &mut buf), "Failed to render layout");

            #[derive(RustcEncodable, Debug)]
            struct Body {
                body: String
            }

            let body = Body {
                body: try_t!(String::from_utf8(buf), "Template was not valid utf8")
            };

            let adjusted_layout = &*data.adjust_layout_path(layout.as_ref());

            // render buffer as body of layout into output stream
            cached_compile(adjusted_layout, data, |template| {
                let template = try_t!(template);
                t.write(|mut writer| template.render(&mut writer, &body))
            })
        })
    } else {
        cached_compile(inner, data, |template| {
            let template = try_t!(template);
            t.write(|writer| f(template, writer))
        })
    }
}

fn cached_compile<D, F, R>(path: &Path, data: &D, handle: F) -> R
where D: TemplateSupport,
      F: FnOnce(Result<&Template, CompileError>) -> R {
    let compile = |path: &Path| {
        mustache::compile_path(path)
            .map_err(|e| format!("Failed to compile template '{}': {:?}", path.display(), e))
    };

    if let Some(cache) = data.cache() {
        return cache.handle(path, handle, compile);
    }

    let template = compile(path);

    match template {
        Ok(ref template) => handle(Ok(template)),
        Err(e) => handle(Err(e)),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::cell::Cell;
    use std::fmt::Debug;
    use std::io::Write;
    use mustache::{self, Template};

    use super::super::*;
    use super::RenderSteps;

    struct Foo {
        use_cache: bool,
        cache: FooCacher,
    }

    impl Foo {
        fn new() -> Foo {
            Foo {
                use_cache: true,
                cache: FooCacher::new(),
            }
        }
    }

    impl<'a> RenderSteps<'a> for &'a Foo {
        type Result = Result<String, CompileError>;
        type ServerData = Foo;

        fn data(&self) -> &'a Self::ServerData {
            self
        }

        fn error(self, e: CompileError) -> Self::Result {
            Err(e)
        }

        fn write<F, E>(self, f: F) -> Self::Result
        where F: FnOnce(&mut Write) -> Result<(), E>,
              E: Debug {
            let mut s = vec![];
            f(&mut s).unwrap();
            Ok(String::from_utf8(s).unwrap())
        }
    }

    struct FooCacher {
        called: Cell<usize>,
        fake_cache_hit: bool,
    }

    impl FooCacher {
        fn new() -> FooCacher {
            FooCacher {
                called: Cell::new(0),
                fake_cache_hit: false,
            }
        }
    }

    impl TemplateSupport for Foo {
        type Cache = FooCacher;

        fn cache(&self) -> Option<&Self::Cache> {
            if self.use_cache {
                Some(&self.cache)
            } else {
                None
            }
        }
    }

    impl TemplateCache for FooCacher {
        fn handle<'a, P, F, R>(&self, path: &'a Path, handle: P, on_miss: F) -> R
        where P: FnOnce(Result<&Template, CompileError>) -> R,
              F: FnOnce(&'a Path) -> Result<Template, CompileError> {
            let val = self.called.get();
            self.called.set(val + 1);

            let template = if self.fake_cache_hit {
                mustache::compile_str("")
            } else {
                match on_miss(path) {
                    Ok(template) => template,
                    Err(e) => return handle(Err(e)),
                }
            };

            handle(Ok(&template))
        }
    }

    mod cache {
        use mustache::Template;
        use std::path::Path;

        use super::Foo;
        use super::super::render;

        fn with_template<F>(path: &Path, data: &Foo, f: F)
        where F: FnOnce(&Template) {
            render(data, path, |template, _| {
                f(template);
                Ok(())
            }).unwrap();
        }

        #[test]
        fn called() {
            let path = Path::new("examples/assets/my_template");
            let data = Foo::new();

            with_template(&path, &data, |_| ());
            assert_eq!(data.cache.called.get(), 1);
            with_template(&path, &data, |_| ());
            assert_eq!(data.cache.called.get(), 2);
        }

        #[test]
        fn used() {
            let path = Path::new("fake_file");
            let mut data = Foo::new();

            data.cache.fake_cache_hit = true;
            // This would try to compile the fake path if the cache doesn't pretend to hit.
            with_template(&path, &data, |_| ());
        }

        #[test]
        #[should_panic(expected = "No such file or directory")]
        fn sanity() {
            let path = Path::new("fake_file");
            let mut data = Foo::new();

            data.cache.fake_cache_hit = false;
            // If this doesn't panic, then the `cache_used` test isn't actually doing a
            // valid test.
            render(&data, &path, |_, _| Ok(())).unwrap();
        }

        #[test]
        fn ignored_when_none() {
            let path = Path::new("examples/assets/my_template");
            let mut data = Foo::new();
            data.use_cache = false;

            with_template(&path, &data, |_| ());
            with_template(&path, &data, |_| ());
            with_template(&path, &data, |_| ());
            with_template(&path, &data, |_| ());
            with_template(&path, &data, |_| ());
            with_template(&path, &data, |_| ());
            with_template(&path, &data, |_| ());
            with_template(&path, &data, |_| ());
            with_template(&path, &data, |_| ());
            with_template(&path, &data, |_| ());
            with_template(&path, &data, |_| ());
            with_template(&path, &data, |_| ());

            assert_eq!(data.cache.called.get(), 0);
        }
    }
}
