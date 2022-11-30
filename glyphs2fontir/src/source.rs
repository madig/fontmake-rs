use std::{path::PathBuf, collections::HashSet};

use fontir::source::{Paths, Source, Input, Work};
use glyphstool::Font;

pub struct GlyphsIrSource {
    glyphs_file: PathBuf,
    ir_paths: Paths,
}

impl GlyphsIrSource {
    pub fn new(glyphs_file: PathBuf, ir_paths: Paths) -> GlyphsIrSource {
        GlyphsIrSource {
            glyphs_file,
            ir_paths,
        }
    }
}

impl Source for GlyphsIrSource {
    fn inputs(&mut self) -> Result<Input, fontir::error::Error> {
        let _font = Font::load(&self.glyphs_file)
            .map_err(|msg| fontir::error::Error::ParseError(self.glyphs_file.clone(), msg))?;
        Ok(Input {
            ..Default::default()
        })
    }

    fn create_glyph_ir_work(
        &self,
        _glyph_names: &HashSet<&str>,
        _input: &Input,
    ) -> Result<Vec<Box<dyn Work<()>>>, fontir::error::Error> {
        todo!()
    }

}