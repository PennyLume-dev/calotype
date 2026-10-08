//! Bridge contract: bad input is an error, never a panic. Real-photo tests run when
//! `CALOTYPE_SAMPLES` names a folder holding `pixel6pro.dng` (CC0, raw.pixls.us); otherwise they skip.

use std::collections::HashMap;
use std::path::PathBuf;

use calotype_bridge::Editor;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("calotype-bridge-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn file(name: &str, bytes: &[u8]) -> String {
    let p = scratch(name);
    std::fs::write(&p, bytes).unwrap();
    p.to_string_lossy().into_owned()
}

fn sample() -> Option<String> {
    let p = PathBuf::from(std::env::var("CALOTYPE_SAMPLES").ok()?).join("pixel6pro.dng");
    p.exists().then(|| p.to_string_lossy().into_owned())
}

#[test]
fn corrupt_files_are_errors() {
    let e = Editor::new();
    let noise: Vec<u8> = (0..65_536u32).map(|i| (i.wrapping_mul(2_654_435_761) >> 13) as u8).collect();
    // little-endian TIFF header pointing its first IFD past the end of the file
    let tiff = [b'I', b'I', 42, 0, 0xff, 0xff, 0xff, 0x7f, 1, 2, 3];
    for (name, bytes) in [("empty.dng", &[][..]), ("noise.dng", &noise[..]), ("bad-ifd.dng", &tiff[..]), ("noise.arw", &noise[..])] {
        assert!(e.import(file(name, bytes)).is_err(), "{name} imported");
    }
    assert!(e.import(scratch("missing.dng").to_string_lossy().into_owned()).is_err());
    assert!(e.list().is_empty());
}

#[test]
fn unknown_ids_and_controls_are_errors() {
    let e = Editor::new();
    assert!(e.preview(42, 512).is_err());
    assert!(e.preview(42, 0).is_err());
    assert!(e.set_controls(42, HashMap::from([("light.exposure".into(), 1.0)])).is_err());
    assert!(e.export(42, scratch("x.jpg").to_string_lossy().into_owned(), None, 90).is_err());
    assert!(!e.undo().unwrap(), "nothing to undo yet");
}

#[test]
fn truncated_raw_is_an_error_or_renders() {
    let Some(path) = sample() else { return };
    let bytes = std::fs::read(path).unwrap();
    for cut in [16, 4096, bytes.len() / 3] {
        let e = Editor::new();
        // Either import refuses it or the render reports an error: no panic either way.
        if let Ok(id) = e.import(file(&format!("cut{cut}.dng"), &bytes[..cut])) {
            let _ = e.preview(id, 256);
        }
    }
}

#[test]
fn import_preview_edit_undo_export() {
    let Some(path) = sample() else { return };
    let e = Editor::new();
    let id = e.import(path.clone()).unwrap();
    assert_eq!(e.import(path).unwrap(), id, "re-import returns the existing photo");
    let list = e.list();
    assert_eq!(list.len(), 1);
    assert!(!list[0].edited);

    let p = e.preview(id, 512).unwrap();
    assert_eq!(p.width.max(p.height), 512);
    assert_eq!(&p.jpeg[..2], &[0xff, 0xd8], "JPEG magic");

    e.set_controls(id, HashMap::from([("light.exposure".into(), 1.0)])).unwrap();
    assert!(e.list()[0].edited);
    assert!(e.set_controls(id, HashMap::from([("no.such.control".into(), 1.0)])).is_err());
    let brighter = e.preview(id, 512).unwrap();
    assert_ne!(brighter.jpeg, p.jpeg);

    assert!(e.undo().unwrap());
    assert!(!e.list()[0].edited);

    let out = scratch("export.jpg");
    let r = e.export(id, out.to_string_lossy().into_owned(), Some(800), 90).unwrap();
    assert_eq!(r.width.max(r.height), 800);
    assert!(std::fs::metadata(out).unwrap().len() > 10_000);
}
