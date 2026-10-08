//! Calotype's bridge: the few calls the Android UI needs, on top of the engine's command registry.
//!
//! Every call returns `Result`; bad input (corrupt file, unknown photo or control) is an error the
//! UI can show, never a panic. UniFFI also turns an escaped panic into a Kotlin exception.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use lightcraft_engine::Session;
use lightcraft_engine::catalog::PhotoId;
use lightcraft_engine::export::{ExportFormat, ExportOptions, Exported, export_photo, write_file};
use serde_json::{Value, json};

uniffi::setup_scaffolding!();

/// Render threads; capped to spare battery and leave cores for the UI.
const MAX_THREADS: usize = 4;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum BridgeError {
    // Not `message`: that clashes with Throwable.message in the Kotlin bindings.
    #[error("{reason}")]
    Failed { reason: String },
}

fn fail(reason: impl ToString) -> BridgeError {
    BridgeError::Failed { reason: reason.to_string() }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct PhotoInfo {
    pub id: u64,
    pub file_name: String,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub edited: bool,
}

/// An encoded JPEG and its pixel size.
#[derive(Debug, Clone, uniffi::Record)]
pub struct Rendered {
    pub width: u32,
    pub height: u32,
    pub jpeg: Vec<u8>,
}

#[derive(uniffi::Object)]
pub struct Editor {
    session: Mutex<Session>,
}

impl Default for Editor {
    fn default() -> Self {
        // A second Editor finds the pool already built; that is fine.
        let threads = std::thread::available_parallelism().map_or(2, |n| n.get()).min(MAX_THREADS);
        let _ = rayon::ThreadPoolBuilder::new().num_threads(threads).build_global();
        Editor { session: Mutex::new(Session::new().with_fs()) }
    }
}

impl Editor {
    fn session(&self) -> MutexGuard<'_, Session> {
        self.session.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn render(s: &mut Session, id: u64, params: Value) -> Result<Exported, BridgeError> {
        let id = Self::known(s, id)?;
        let mut o = ExportOptions::from_json(&params);
        o.format = ExportFormat::parse("jpg").ok_or_else(|| fail("jpeg encoder missing"))?;
        export_photo(s, id, &o, 1).map_err(fail)
    }

    fn known(s: &Session, id: u64) -> Result<PhotoId, BridgeError> {
        s.catalog.photo(PhotoId(id)).map(|p| p.id).ok_or_else(|| fail(format!("no photo {id}")))
    }

    fn select(s: &mut Session, id: PhotoId) -> Result<(), BridgeError> {
        s.execute("library.select", &json!({"ids": [id.0], "active": id.0})).map(|_| ()).map_err(fail)
    }
}

#[uniffi::export]
impl Editor {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Imports one file by path (the app copies picked files to its own storage first) and makes it
    /// the active photo. Returns the photo id; a file already imported returns its existing id.
    pub fn import(&self, path: String) -> Result<u64, BridgeError> {
        let mut s = self.session();
        let r = s.execute("library.import", &json!({"paths": [path]})).map_err(fail)?;
        let id = r
            .pointer("/imported/0")
            .or_else(|| r.pointer("/duplicates/0/existing"))
            .and_then(Value::as_u64)
            .ok_or_else(|| fail(r.pointer("/failed/0/1").and_then(Value::as_str).unwrap_or("not a readable photo")))?;
        Self::select(&mut s, PhotoId(id))?;
        Ok(id)
    }

    pub fn list(&self) -> Vec<PhotoInfo> {
        let s = self.session();
        s.catalog
            .photos()
            .filter(|p| !p.deleted)
            .map(|p| PhotoInfo {
                id: p.id.0,
                file_name: p.file_name.clone(),
                format: p.format.clone(),
                width: p.width,
                height: p.height,
                edited: p.is_edited(),
            })
            .collect()
    }

    /// The developed photo as a JPEG whose long edge is at most `long_edge` pixels.
    pub fn preview(&self, id: u64, long_edge: u32) -> Result<Rendered, BridgeError> {
        if long_edge == 0 {
            return Err(fail("preview size must be above 0"));
        }
        let e = Self::render(&mut self.session(), id, json!({"quality": 85, "longEdge": long_edge}))?;
        Ok(Rendered { width: e.width as u32, height: e.height as u32, jpeg: e.bytes })
    }

    /// Sets develop controls by id (`light.exposure`, `light.highlights`, …) as one undo step.
    pub fn set_controls(&self, id: u64, values: HashMap<String, f64>) -> Result<(), BridgeError> {
        if values.is_empty() {
            return Ok(());
        }
        let mut s = self.session();
        let id = Self::known(&s, id)?;
        Self::select(&mut s, id)?;
        s.execute("develop.set", &json!({"values": values, "ids": [id.0]})).map(|_| ()).map_err(fail)
    }

    /// Undoes the last step. False when there was nothing to undo.
    pub fn undo(&self) -> Result<bool, BridgeError> {
        let mut s = self.session();
        if s.undo.is_empty() {
            return Ok(false);
        }
        s.execute("edit.undo", &Value::Null).map(|_| true).map_err(fail)
    }

    /// Writes the developed photo as a JPEG to `path`; `long_edge` `None` = full size.
    pub fn export(&self, id: u64, path: String, long_edge: Option<u32>, quality: u8) -> Result<Rendered, BridgeError> {
        let mut p = json!({"quality": quality.clamp(1, 100)});
        if let Some(n) = long_edge.filter(|n| *n > 0) {
            p["longEdge"] = json!(n);
        }
        let e = Self::render(&mut self.session(), id, p)?;
        write_file(&path, &e.bytes).map_err(fail)?;
        // The pixels are on disk; don't send them back over the bridge.
        Ok(Rendered { width: e.width as u32, height: e.height as u32, jpeg: Vec::new() })
    }
}
