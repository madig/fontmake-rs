//! Generates a [gvar](https://learn.microsoft.com/en-us/typography/opentype/spec/gvar) table.

use fontdrasil::orchestration::Work;
use log::warn;

use crate::{
    error::Error,
    orchestration::{BeWork, Context},
};

struct GvarWork {}

pub fn create_gvar_work() -> Box<BeWork> {
    Box::new(GvarWork {})
}

impl Work<Context, Error> for GvarWork {
    /// Generate [gvar](https://learn.microsoft.com/en-us/typography/opentype/spec/gvar)
    fn exec(&self, _context: &Context) -> Result<(), Error> {
        warn!("gvar not actually implemented");
        Ok(())
    }
}
