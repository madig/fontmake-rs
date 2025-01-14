//! Feature binary compilation.

use std::{
    ffi::{OsStr, OsString},
    fmt::Display,
    fs,
    sync::Arc,
};

use fea_rs::{
    parse::{SourceLoadError, SourceResolver},
    Compiler, GlyphMap, GlyphName as FeaRsGlyphName,
};
use fontir::{ir::Features, orchestration::Flags};
use log::{debug, error, trace, warn};
use write_fonts::FontBuilder;

use fontdrasil::orchestration::Work;

use crate::{
    error::Error,
    orchestration::{BeWork, Context},
};

pub struct FeatureWork {}

// I did not want to make a struct
// I did not want to clone the content
// I do not like this construct
// I do find the need to lament
struct InMemoryResolver {
    content_path: OsString,
    content: Arc<str>,
}

impl SourceResolver for InMemoryResolver {
    fn get_contents(&self, path: &OsStr) -> Result<Arc<str>, SourceLoadError> {
        if path == &*self.content_path {
            return Ok(self.content.clone());
        }
        Err(SourceLoadError::new(
            path.to_os_string(),
            NotSupportedError::new(),
        ))
    }
}

#[derive(Debug)]
struct NotSupportedError {}

impl NotSupportedError {
    fn new() -> NotSupportedError {
        NotSupportedError {}
    }
}

impl std::error::Error for NotSupportedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl Display for NotSupportedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Not supported")?;
        Ok(())
    }
}

impl FeatureWork {
    pub fn create() -> Box<BeWork> {
        Box::new(FeatureWork {})
    }

    fn compile(&self, features: &Features, glyph_order: GlyphMap) -> Result<FontBuilder, Error> {
        let root_path = if let Features::File(file) = features {
            OsString::from(file)
        } else {
            OsString::new()
        };
        let mut compiler = Compiler::new(root_path.clone(), &glyph_order);
        if let Features::Memory(fea_content) = features {
            let resolver = InMemoryResolver {
                content_path: root_path,
                content: Arc::from(fea_content.as_str()),
            };
            compiler = compiler.with_resolver(resolver);
        }
        compiler
            .compile()
            .map_err(Error::FeaCompileError)?
            .assemble(&glyph_order, Default::default())
            .map_err(Error::FeaAssembleError)
    }
}

fn write_debug_fea(context: &Context, is_error: bool, why: &str, fea_content: &str) {
    let debug_file = context.debug_dir().join("glyphs.fea");
    match fs::write(&debug_file, fea_content) {
        Ok(..) => {
            if is_error {
                warn!("{}; fea written to {:?}", why, debug_file)
            } else {
                debug!("fea written to {:?}", debug_file);
            }
        }
        Err(e) => error!("{}; failed to write fea to {:?}: {}", why, debug_file, e),
    };
}

impl Work<Context, Error> for FeatureWork {
    fn exec(&self, context: &Context) -> Result<(), Error> {
        let features = context.ir.get_features();
        if let Features::Empty = *features {
            // set a default in place so subsequent compiles skip this step
            trace!("No fea file, dull compile");
            context.set_features(FontBuilder::default());
            return Ok(());
        }
        let glyph_order = &context.ir.get_final_static_metadata().glyph_order;
        if glyph_order.is_empty() {
            warn!("Glyph order is empty; feature compile improbable");
        }
        let glyph_map = glyph_order
            .iter()
            .map(|n| Into::<FeaRsGlyphName>::into(n.as_str()))
            .collect();

        let result = self.compile(&features, glyph_map);
        if result.is_err() || context.flags.contains(Flags::EMIT_DEBUG) {
            if let Features::Memory(fea_content) = &*features {
                write_debug_fea(context, result.is_err(), "compile failed", fea_content);
            }
        }
        let font = result?;
        context.set_features(font);
        Ok(())
    }
}
