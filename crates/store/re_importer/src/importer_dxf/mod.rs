//! Rerun importer for [DXF](https://en.wikipedia.org/wiki/AutoCAD_DXF) CAD files.

mod domain;
mod emit;
mod parse;

use std::path::{Path, PathBuf};

use anyhow::Context as _;
use crossbeam::channel::Sender;
use re_chunk::{Chunk, EntityPath};
use re_log_types::{EntityPathPart, TimePoint};

use crate::{ImportedData, Importer, ImporterError};

pub use domain::Drawing;

fn is_dxf_file(path: impl AsRef<Path>) -> bool {
    path.as_ref()
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("dxf"))
}

fn drawing_root_path(filepath: &Path, entity_path_prefix: Option<&EntityPath>) -> EntityPath {
    let filename = filepath
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "drawing".to_owned());
    let safe_filename = emit::sanitize_path_part(&filename);

    match entity_path_prefix {
        Some(prefix) => prefix.clone() / EntityPathPart::new(safe_filename),
        None => EntityPath::from_single_string(safe_filename),
    }
}

fn emit_dxf(
    emit: &mut dyn FnMut(Chunk),
    drawing: &Drawing,
    filepath: &Path,
    entity_path_prefix: Option<&EntityPath>,
    timepoint: &TimePoint,
) -> anyhow::Result<()> {
    let root = drawing_root_path(filepath, entity_path_prefix);
    emit::emit_drawing(emit, drawing, root, timepoint)
}

/// An [`Importer`] for [DXF](https://en.wikipedia.org/wiki/AutoCAD_DXF) CAD drawings.
pub struct DxfImporter;

impl Importer for DxfImporter {
    fn name(&self) -> crate::ImporterName {
        "rerun.importers.Dxf".to_owned()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn import_from_path(
        &self,
        settings: &crate::ImporterSettings,
        filepath: PathBuf,
        tx: Sender<ImportedData>,
    ) -> Result<(), ImporterError> {
        if !is_dxf_file(&filepath) {
            return Err(ImporterError::Incompatible(filepath));
        }

        re_tracing::profile_function!(filepath.display().to_string());

        let drawing =
            parse::load_dxf(&filepath).with_context(|| format!("Path: {}", filepath.display()))?;

        let store_id = settings.opened_store_id_or_recommended();
        let timepoint = settings.timepoint.clone().unwrap_or_default();
        let mut send_error = None;
        let mut emit = |chunk| {
            if send_error.is_none() {
                send_error = re_quota_channel::send_crossbeam(
                    &tx,
                    ImportedData::Chunk(Self.name(), store_id.clone(), chunk),
                )
                .err();
            }
        };

        emit_dxf(
            &mut emit,
            &drawing,
            &filepath,
            settings.entity_path_prefix.as_ref(),
            &timepoint,
        )
        .with_context(|| "Failed to load DXF file!")?;

        if let Some(err) = send_error {
            return Err(anyhow::anyhow!(err.to_string()).into());
        }

        Ok(())
    }

    fn import_from_file_contents(
        &self,
        settings: &crate::ImporterSettings,
        filepath: PathBuf,
        contents: std::borrow::Cow<'_, [u8]>,
        tx: Sender<ImportedData>,
    ) -> Result<(), ImporterError> {
        if !is_dxf_file(&filepath) {
            return Err(ImporterError::Incompatible(filepath));
        }

        re_tracing::profile_function!(filepath.display().to_string());

        let drawing = parse::load_dxf_bytes(&contents)
            .with_context(|| format!("Path: {}", filepath.display()))?;

        let store_id = settings.opened_store_id_or_recommended();
        let timepoint = settings.timepoint.clone().unwrap_or_default();
        let mut send_error = None;
        let mut emit = |chunk| {
            if send_error.is_none() {
                send_error = re_quota_channel::send_crossbeam(
                    &tx,
                    ImportedData::Chunk(Self.name(), store_id.clone(), chunk),
                )
                .err();
            }
        };

        emit_dxf(
            &mut emit,
            &drawing,
            &filepath,
            settings.entity_path_prefix.as_ref(),
            &timepoint,
        )
        .with_context(|| "Failed to load DXF file!")?;

        if let Some(err) = send_error {
            return Err(anyhow::anyhow!(err.to_string()).into());
        }

        Ok(())
    }
}
