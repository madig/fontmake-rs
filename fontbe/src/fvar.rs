//! Generates a [fvar](https://learn.microsoft.com/en-us/typography/opentype/spec/fvar) table.

use std::collections::HashMap;

use fontdrasil::orchestration::Work;
use fontir::ir::StaticMetadata;
use log::trace;
use read_fonts::types::MajorMinor;
use write_fonts::tables::fvar::{AxisInstanceArrays, Fvar, VariationAxisRecord};

use crate::{
    error::Error,
    orchestration::{BeWork, Context},
};

const HIDDEN_AXIS: u16 = 0x0001;

struct FvarWork {}

pub fn create_fvar_work() -> Box<BeWork> {
    Box::new(FvarWork {})
}

fn generate_fvar(static_metadata: &StaticMetadata) -> Option<Fvar> {
    // Guard clause: don't produce fvar for a static font
    if static_metadata.variable_axes.is_empty() {
        trace!("Skip fvar; this is not a variable font");
        return None;
    }

    let reverse_names: HashMap<_, _> = static_metadata
        .names
        .iter()
        .map(|(key, name)| (name, key.name_id))
        .collect();

    let axes_and_instances = AxisInstanceArrays::new(
        static_metadata
            .variable_axes
            .iter()
            .map(|ir_axis| {
                let mut var = VariationAxisRecord {
                    axis_tag: ir_axis.tag,
                    min_value: ir_axis.min.into(),
                    default_value: ir_axis.default.into(),
                    max_value: ir_axis.max.into(),
                    axis_name_id: *reverse_names.get(&ir_axis.name).unwrap(),
                    ..Default::default()
                };
                if ir_axis.hidden {
                    var.flags |= HIDDEN_AXIS;
                }
                var
            })
            .collect(),
        Vec::new(),
    );

    let axis_count = axes_and_instances.axes.len().try_into().unwrap();
    let instance_count = axes_and_instances.instances.len().try_into().unwrap();

    Some(Fvar::new(
        MajorMinor::VERSION_1_0,
        axes_and_instances,
        axis_count,
        instance_count,
    ))
}

impl Work<Context, Error> for FvarWork {
    /// Generate [fvar](https://learn.microsoft.com/en-us/typography/opentype/spec/fvar)
    fn exec(&self, context: &Context) -> Result<(), Error> {
        if let Some(fvar) = generate_fvar(&context.ir.get_init_static_metadata()) {
            context.set_fvar(fvar);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{cmp, str::FromStr};

    use fontir::{
        coords::{CoordConverter, DesignCoord, UserCoord},
        ir::{Axis, StaticMetadata},
    };
    use read_fonts::types::Tag;

    use super::generate_fvar;

    fn axis(min: f32, default: f32, max: f32) -> Axis {
        let mut mappings = Vec::new();
        if min < default {
            mappings.push((UserCoord::new(min), DesignCoord::new(min / 10.0)));
        }
        mappings.push((UserCoord::new(default), DesignCoord::new(default / 10.0)));
        if max > default {
            mappings.push((UserCoord::new(max), DesignCoord::new(max / 10.0)));
        }
        let default_idx = cmp::min(mappings.len() - 1, 1);
        Axis {
            name: "Test".to_string(),
            tag: Tag::from_str("TEST").unwrap(),
            min: UserCoord::new(min),
            default: UserCoord::new(default),
            max: UserCoord::new(max),
            hidden: false,
            converter: CoordConverter::new(mappings, default_idx),
        }
    }

    fn create_static_metadata(axes: &[Axis]) -> StaticMetadata {
        StaticMetadata::new(
            1000,
            Default::default(),
            axes.to_vec(),
            Default::default(),
            Default::default(),
        )
        .unwrap()
    }

    #[test]
    fn no_fvar_for_no_axes() {
        let static_metadata = create_static_metadata(&[]);
        let fvar = generate_fvar(&static_metadata);
        assert!(fvar.is_none());
    }

    #[test]
    fn no_fvar_for_point_axes() {
        let static_metadata = create_static_metadata(&[axis(400.0, 400.0, 400.0)]);
        let fvar = generate_fvar(&static_metadata);
        assert!(fvar.is_none());
    }

    #[test]
    fn fvar_includes_only_variable_axes() {
        let static_metadata =
            create_static_metadata(&[axis(400.0, 400.0, 700.0), axis(400.0, 400.0, 400.0)]);
        let fvar = generate_fvar(&static_metadata).unwrap();
        assert_eq!(
            vec![(400.0, 400.0, 700.0),],
            fvar.axis_instance_arrays
                .axes
                .iter()
                .map(|var| (
                    var.min_value.to_f64(),
                    var.default_value.to_f64(),
                    var.max_value.to_f64()
                ))
                .collect::<Vec<_>>()
        );
    }
}
