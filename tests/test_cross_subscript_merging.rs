//! Regression tests for cross-subscript span merging.

use pdf_oxide::document::PdfDocument;
use pdf_oxide::writer::{PageBuilder, PdfWriter};

fn put(page: &mut PageBuilder<'_>, text: &str, x: f32, y: f32, font: &str, size: f32) {
    page.add_text(text, x, y, font, size);
    page.draw_rect(0.0, 0.0, 0.0, 0.0);
}

fn build_and_extract(build_fn: impl FnOnce(&mut PdfWriter)) -> String {
    let mut writer = PdfWriter::new();
    build_fn(&mut writer);
    let bytes = writer.finish().expect("build PDF");
    let mut doc = PdfDocument::from_bytes(bytes).expect("open PDF");
    doc.extract_text(0).expect("extract page 0")
}

#[test]
fn h2o_synthetic_extracts_correctly() {
    let out = build_and_extract(|w| {
        let mut page = w.add_letter_page();
        put(&mut page, "H", 100.0, 200.0, "Helvetica", 12.0);
        put(&mut page, "2", 107.0, 197.0, "Helvetica", 8.0);
        put(&mut page, "O", 108.0, 200.0, "Helvetica", 12.0);
    });

    assert_eq!(out.trim_end(), "H2O", "got {:?}", out.trim_end());
}

#[test]
fn tight_kerned_pair_still_merges() {
    let out = build_and_extract(|w| {
        let mut page = w.add_letter_page();
        put(&mut page, "A", 100.0, 200.0, "Helvetica", 12.0);
        put(&mut page, "B", 109.0, 200.0, "Helvetica", 12.0);
    });

    let trimmed = out.trim_end();
    assert!(!trimmed.contains('\n'), "got {:?}", trimmed);
    assert!(trimmed.contains('A') && trimmed.contains('B'), "got {:?}", trimmed);
}

#[test]
fn nearby_non_interposed_glyph_does_not_block_merge() {
    let out = build_and_extract(|w| {
        let mut page = w.add_letter_page();
        put(&mut page, "A", 100.0, 200.0, "Helvetica", 12.0);
        put(&mut page, "B", 109.0, 200.0, "Helvetica", 12.0);
        put(&mut page, "9", 200.0, 197.0, "Helvetica", 8.0);
    });

    let trimmed = out.trim_end();
    let a_pos = trimmed.find('A').expect("A present");
    let b_pos = trimmed.find('B').expect("B present");
    let between = &trimmed[a_pos..b_pos];

    assert!(a_pos < b_pos, "got {:?}", trimmed);
    assert!(!between.contains('\n'), "got {:?}", trimmed);
    assert!(trimmed.contains('9'), "got {:?}", trimmed);
}
