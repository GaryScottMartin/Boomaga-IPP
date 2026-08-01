//! Content-preserving PDF assembly for imposed booklet sides.

use crate::{Error, Result};
use qpdf::{
    QPdf, QPdfArray, QPdfDictionary, QPdfObject, QPdfObjectLike, QPdfObjectType, QPdfScalar,
};
use std::path::Path;

/// Assemble two-up booklet sides into a standalone PDF.
///
/// Each entry in `sides` is one output page in printer order. Source page
/// indices are zero-based and `None` produces an empty half-sheet.
pub fn assemble_booklet_pdf(
    source_path: &Path,
    output_path: &Path,
    sides: &[[Option<usize>; 2]],
) -> Result<()> {
    if sides.is_empty() {
        return Err(Error::Validation(
            "Booklet assembly requires at least one output side".into(),
        ));
    }

    let source = QPdf::read(source_path).map_err(pdf_error)?;
    let source_page_count = source.get_num_pages().map_err(pdf_error)? as usize;
    if source_page_count == 0 {
        return Err(Error::Validation(
            "Cannot assemble a booklet from an empty PDF".into(),
        ));
    }
    for page_index in sides.iter().flatten().flatten() {
        if *page_index >= source_page_count {
            return Err(Error::Validation(format!(
                "Booklet source page {} exceeds PDF page count {source_page_count}",
                page_index + 1
            )));
        }
    }

    let first_page = source
        .get_page(0)
        .ok_or_else(|| Error::Pdf("Unable to read the first PDF page".into()))?;
    let first_geometry = page_geometry(&first_page)?;
    let output_width = first_geometry.height;
    let output_height = first_geometry.width;
    let output = QPdf::empty();

    for slots in sides {
        let xobjects = output.new_dictionary();
        let mut operators = String::new();

        for (slot_index, source_index) in slots.iter().enumerate() {
            let Some(source_index) = source_index else {
                continue;
            };
            let page = source.get_page(*source_index as u32).ok_or_else(|| {
                Error::Pdf(format!("Unable to read source page {}", source_index + 1))
            })?;
            let form_name = format!("/B{}", slot_index + 1);
            let (form, transform) =
                create_page_form(&output, &page, slot_index, output_width, output_height)?;
            xobjects.set(&form_name, form.into_indirect());
            let [a, b, c, d, e, f] = transform.normalize;
            operators.push_str(&format!(
                "q {:.8} 0 0 {:.8} {:.8} {:.8} cm \
                 {a:.8} {b:.8} {c:.8} {d:.8} {e:.8} {f:.8} cm {form_name} Do Q\n",
                transform.scale, transform.scale, transform.x, transform.y
            ));
        }

        let resources = output.new_dictionary();
        resources.set("/XObject", xobjects);
        let contents = output.new_stream(operators.as_bytes());
        let media_box = output
            .parse_object(&format!("[0 0 {output_width:.8} {output_height:.8}]"))
            .map_err(pdf_error)?;
        let page = output.new_dictionary();
        page.set("/Type", output.new_name("/Page"));
        page.set("/MediaBox", media_box);
        page.set("/Resources", resources);
        page.set("/Contents", contents.into_indirect());
        output
            .add_page(page.into_indirect(), false)
            .map_err(pdf_error)?;
    }

    output.check_pdf().map_err(pdf_error)?;
    output.writer().write(output_path).map_err(pdf_error)
}

/// Assemble source pages into deterministic N-up sheets in row-major or
/// column-major fill order. Source indices are zero-based.
pub fn assemble_n_up_pdf(
    source_path: &Path,
    output_path: &Path,
    pages: &[usize],
    pages_per_sheet: u8,
    vertical_fill: bool,
) -> Result<()> {
    let (columns, rows) = n_up_grid(pages_per_sheet)?;
    if pages.is_empty() {
        return Err(Error::Validation(
            "N-up assembly requires at least one source page".into(),
        ));
    }

    let source = QPdf::read(source_path).map_err(pdf_error)?;
    let source_page_count = source.get_num_pages().map_err(pdf_error)? as usize;
    for page_index in pages {
        if *page_index >= source_page_count {
            return Err(Error::Validation(format!(
                "N-up source page {} exceeds PDF page count {source_page_count}",
                page_index + 1
            )));
        }
    }

    let first_page = source
        .get_page(pages[0] as u32)
        .ok_or_else(|| Error::Pdf("Unable to read the first selected PDF page".into()))?;
    let first_geometry = page_geometry(&first_page)?;
    let portrait_width = first_geometry.width.min(first_geometry.height);
    let portrait_height = first_geometry.width.max(first_geometry.height);
    let (output_width, output_height) = if matches!(pages_per_sheet, 2 | 6 | 8) {
        (portrait_height, portrait_width)
    } else {
        (portrait_width, portrait_height)
    };
    let output = QPdf::empty();

    for sheet_pages in pages.chunks(usize::from(pages_per_sheet)) {
        let xobjects = output.new_dictionary();
        let mut operators = String::new();
        for (input_index, source_index) in sheet_pages.iter().enumerate() {
            let slot_index = n_up_slot(input_index, columns, rows, vertical_fill);
            let page = source.get_page(*source_index as u32).ok_or_else(|| {
                Error::Pdf(format!("Unable to read source page {}", source_index + 1))
            })?;
            let form_name = format!("/N{}", input_index + 1);
            let (form, transform) = create_n_up_page_form(
                &output,
                &page,
                slot_index,
                columns,
                rows,
                output_width,
                output_height,
            )?;
            xobjects.set(&form_name, form.into_indirect());
            let [a, b, c, d, e, f] = transform.normalize;
            operators.push_str(&format!(
                "q {:.8} 0 0 {:.8} {:.8} {:.8} cm {a:.8} {b:.8} {c:.8} {d:.8} {e:.8} {f:.8} cm {form_name} Do Q\n",
                transform.scale, transform.scale, transform.x, transform.y
            ));
        }

        let resources = output.new_dictionary();
        resources.set("/XObject", xobjects);
        let contents = output.new_stream(operators.as_bytes());
        let media_box = output
            .parse_object(&format!("[0 0 {output_width:.8} {output_height:.8}]"))
            .map_err(pdf_error)?;
        let page = output.new_dictionary();
        page.set("/Type", output.new_name("/Page"));
        page.set("/MediaBox", media_box);
        page.set("/Resources", resources);
        page.set("/Contents", contents.into_indirect());
        output
            .add_page(page.into_indirect(), false)
            .map_err(pdf_error)?;
    }

    output.check_pdf().map_err(pdf_error)?;
    output.writer().write(output_path).map_err(pdf_error)
}

fn n_up_grid(pages_per_sheet: u8) -> Result<(usize, usize)> {
    match pages_per_sheet {
        1 => Ok((1, 1)),
        2 => Ok((2, 1)),
        4 => Ok((2, 2)),
        6 => Ok((3, 2)),
        8 => Ok((4, 2)),
        value => Err(Error::Validation(format!(
            "Unsupported N-up page count: {value}"
        ))),
    }
}

fn n_up_slot(index: usize, columns: usize, rows: usize, vertical_fill: bool) -> usize {
    if vertical_fill && rows > 1 {
        (index % rows) * columns + index / rows
    } else {
        index
    }
}
/// Rewrite every source page with its inherited `/Rotate` transform embedded
/// in the page content, leaving no rotation metadata for CUPS N-up to reapply.
pub fn normalize_pdf_page_rotations(source_path: &Path, output_path: &Path) -> Result<()> {
    let source = QPdf::read(source_path).map_err(pdf_error)?;
    let source_page_count = source.get_num_pages().map_err(pdf_error)?;
    if source_page_count == 0 {
        return Err(Error::Validation(
            "Cannot normalize rotations in an empty PDF".into(),
        ));
    }

    let output = QPdf::empty();
    for page_index in 0..source_page_count {
        let source_page = source
            .get_page(page_index)
            .ok_or_else(|| Error::Pdf(format!("Unable to read source page {}", page_index + 1)))?;
        let source_box = inherited_page_value(&source_page, "/CropBox")
            .or_else(|| inherited_page_value(&source_page, "/MediaBox"))
            .ok_or_else(|| Error::Pdf("Source PDF page has no MediaBox".into()))?;
        let geometry = page_geometry_from_box(&source_page, &source_box)?;

        let resources = inherited_page_value(&source_page, "/Resources")
            .map(|value| output.copy_from_foreign(value.into_indirect()))
            .unwrap_or_else(|| output.new_dictionary().into());
        let form = output.new_stream(
            source_page
                .get_page_content_data()
                .map_err(pdf_error)?
                .as_ref(),
        );
        let form_dictionary = form.get_dictionary();
        form_dictionary.set("/Type", output.new_name("/XObject"));
        form_dictionary.set("/Subtype", output.new_name("/Form"));
        form_dictionary.set("/FormType", output.new_integer(1));
        form_dictionary.set(
            "/BBox",
            output.copy_from_foreign(source_box.into_indirect()),
        );
        form_dictionary.set("/Resources", resources);

        let xobjects = output.new_dictionary();
        xobjects.set("/P", form.into_indirect());
        let page_resources = output.new_dictionary();
        page_resources.set("/XObject", xobjects);
        let [a, b, c, d, e, f] = geometry.normalize;
        let contents = output
            .new_stream(format!("{a:.8} {b:.8} {c:.8} {d:.8} {e:.8} {f:.8} cm /P Do\n").as_bytes());
        let media_box = output
            .parse_object(&format!(
                "[0 0 {:.8} {:.8}]",
                geometry.width, geometry.height
            ))
            .map_err(pdf_error)?;
        let page = output.new_dictionary();
        page.set("/Type", output.new_name("/Page"));
        page.set("/MediaBox", media_box);
        page.set("/Resources", page_resources);
        page.set("/Contents", contents.into_indirect());
        output
            .add_page(page.into_indirect(), false)
            .map_err(pdf_error)?;
    }

    output.check_pdf().map_err(pdf_error)?;
    output.writer().write(output_path).map_err(pdf_error)
}
#[derive(Debug, Clone, Copy)]
struct Placement {
    scale: f64,
    x: f64,
    y: f64,
    normalize: [f64; 6],
}

#[derive(Debug, Clone, Copy)]
struct PageGeometry {
    width: f64,
    height: f64,
    normalize: [f64; 6],
}

fn create_page_form(
    output: &QPdf,
    page: &QPdfDictionary,
    slot_index: usize,
    output_width: f64,
    output_height: f64,
) -> Result<(qpdf::QPdfStream, Placement)> {
    let source_box = inherited_page_value(page, "/CropBox")
        .or_else(|| inherited_page_value(page, "/MediaBox"))
        .ok_or_else(|| Error::Pdf("Source PDF page has no MediaBox".into()))?;
    let geometry = page_geometry_from_box(page, &source_box)?;

    let resources = inherited_page_value(page, "/Resources")
        .map(|value| output.copy_from_foreign(value.into_indirect()))
        .unwrap_or_else(|| output.new_dictionary().into());
    let bbox = output.copy_from_foreign(source_box.into_indirect());
    let content = page.get_page_content_data().map_err(pdf_error)?;
    let form = output.new_stream(content.as_ref());
    let dictionary = form.get_dictionary();
    dictionary.set("/Type", output.new_name("/XObject"));
    dictionary.set("/Subtype", output.new_name("/Form"));
    dictionary.set("/FormType", output.new_integer(1));
    dictionary.set("/BBox", bbox);
    dictionary.set("/Resources", resources);

    let cell_width = output_width / 2.0;
    let scale = (cell_width / geometry.width).min(output_height / geometry.height);
    let x = slot_index as f64 * cell_width + (cell_width - geometry.width * scale) / 2.0;
    let y = (output_height - geometry.height * scale) / 2.0;

    Ok((
        form,
        Placement {
            scale,
            x,
            y,
            normalize: geometry.normalize,
        },
    ))
}

fn create_n_up_page_form(
    output: &QPdf,
    page: &QPdfDictionary,
    slot_index: usize,
    columns: usize,
    rows: usize,
    output_width: f64,
    output_height: f64,
) -> Result<(qpdf::QPdfStream, Placement)> {
    let source_box = inherited_page_value(page, "/CropBox")
        .or_else(|| inherited_page_value(page, "/MediaBox"))
        .ok_or_else(|| Error::Pdf("Source PDF page has no MediaBox".into()))?;
    let geometry = page_geometry_from_box(page, &source_box)?;
    let resources = inherited_page_value(page, "/Resources")
        .map(|value| output.copy_from_foreign(value.into_indirect()))
        .unwrap_or_else(|| output.new_dictionary().into());
    let form = output.new_stream(page.get_page_content_data().map_err(pdf_error)?.as_ref());
    let dictionary = form.get_dictionary();
    dictionary.set("/Type", output.new_name("/XObject"));
    dictionary.set("/Subtype", output.new_name("/Form"));
    dictionary.set("/FormType", output.new_integer(1));
    dictionary.set(
        "/BBox",
        output.copy_from_foreign(source_box.into_indirect()),
    );
    dictionary.set("/Resources", resources);

    let cell_width = output_width / columns as f64;
    let cell_height = output_height / rows as f64;
    let column = slot_index % columns;
    let row = slot_index / columns;
    let scale = (cell_width / geometry.width).min(cell_height / geometry.height);
    let x = column as f64 * cell_width + (cell_width - geometry.width * scale) / 2.0;
    let y = output_height - (row + 1) as f64 * cell_height
        + (cell_height - geometry.height * scale) / 2.0;

    Ok((
        form,
        Placement {
            scale,
            x,
            y,
            normalize: geometry.normalize,
        },
    ))
}
fn page_geometry(page: &QPdfDictionary) -> Result<PageGeometry> {
    let page_box = inherited_page_value(page, "/CropBox")
        .or_else(|| inherited_page_value(page, "/MediaBox"))
        .ok_or_else(|| Error::Pdf("Source PDF page has no MediaBox".into()))?;
    page_geometry_from_box(page, &page_box)
}

fn page_geometry_from_box(
    page: &QPdfDictionary,
    page_box_object: &QPdfObject,
) -> Result<PageGeometry> {
    let [left, bottom, right, top] = page_box(page_box_object)?;
    let width = right - left;
    let height = top - bottom;
    if width <= 0.0 || height <= 0.0 {
        return Err(Error::Pdf("Source PDF page has an invalid page box".into()));
    }

    let rotation = inherited_page_value(page, "/Rotate")
        .map(|value| {
            if value.get_type() != QPdfObjectType::Integer {
                return Err(Error::Pdf("PDF page Rotate value is not an integer".into()));
            }
            Ok(QPdfScalar::from(value).as_i32().rem_euclid(360))
        })
        .transpose()?
        .unwrap_or(0);

    let (width, height, normalize) = match rotation {
        0 => (width, height, [1.0, 0.0, 0.0, 1.0, -left, -bottom]),
        90 => (height, width, [0.0, -1.0, 1.0, 0.0, -bottom, right]),
        180 => (width, height, [-1.0, 0.0, 0.0, -1.0, right, top]),
        270 => (height, width, [0.0, 1.0, -1.0, 0.0, top, -left]),
        value => {
            return Err(Error::Pdf(format!(
                "Unsupported PDF page rotation: {value} degrees"
            )))
        }
    };

    Ok(PageGeometry {
        width,
        height,
        normalize,
    })
}

fn inherited_page_value(page: &QPdfDictionary, key: &str) -> Option<QPdfObject> {
    if let Some(value) = page.get(key) {
        return Some(value);
    }
    let parent = page.get("/Parent")?;
    let parent: QPdfDictionary = parent.into();
    inherited_page_value(&parent, key)
}

fn page_box(value: &QPdfObject) -> Result<[f64; 4]> {
    if value.get_type() != QPdfObjectType::Array {
        return Err(Error::Pdf("PDF page box is not an array".into()));
    }
    let array: QPdfArray = value.clone().into();
    if array.len() != 4 {
        return Err(Error::Pdf("PDF page box must contain four numbers".into()));
    }

    let mut numbers = [0.0; 4];
    for (index, number) in numbers.iter_mut().enumerate() {
        let value = array
            .get(index)
            .ok_or_else(|| Error::Pdf("PDF page box is incomplete".into()))?;
        if !matches!(
            value.get_type(),
            QPdfObjectType::Integer | QPdfObjectType::Real
        ) {
            return Err(Error::Pdf("PDF page box contains a non-number".into()));
        }
        *number = QPdfScalar::from(value).as_f64();
    }
    Ok(numbers)
}

fn pdf_error(error: qpdf::QPdfError) -> Error {
    Error::Pdf(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use qpdf::StreamDecodeLevel;
    use tempfile::TempDir;

    fn create_source(path: &Path, page_count: usize) {
        let pdf = QPdf::empty();
        for page_number in 1..=page_count {
            let content = pdf.new_stream(format!("BT ({page_number}) Tj ET\n").as_bytes());
            let page = pdf.new_dictionary();
            page.set("/Type", pdf.new_name("/Page"));
            page.set("/MediaBox", pdf.parse_object("[0 0 200 300]").unwrap());
            page.set("/Resources", pdf.new_dictionary());
            if page_number == 2 {
                page.set("/Rotate", pdf.new_integer(90));
            }
            page.set("/Contents", content.into_indirect());
            pdf.add_page(page.into_indirect(), false).unwrap();
        }
        pdf.writer().write(path).unwrap();
    }

    #[test]
    fn assembles_ordered_two_up_sides_and_preserves_blanks() {
        let directory = TempDir::new().unwrap();
        let source_path = directory.path().join("source.pdf");
        let output_path = directory.path().join("booklet.pdf");
        create_source(&source_path, 3);

        assemble_booklet_pdf(
            &source_path,
            &output_path,
            &[[None, Some(0)], [Some(1), Some(2)]],
        )
        .unwrap();

        let output = QPdf::read(&output_path).unwrap();
        output.check_pdf().unwrap();
        assert_eq!(output.get_num_pages().unwrap(), 2);

        let first = output.get_page(0).unwrap();
        let first_content = first.get_page_content_data().unwrap();
        let first_content = String::from_utf8_lossy(first_content.as_ref());
        assert!(!first_content.contains("/B1 Do"));
        assert!(first_content.contains("/B2 Do"));

        let second = output.get_page(1).unwrap();
        let resources: QPdfDictionary = second.get("/Resources").unwrap().into();
        let xobjects: QPdfDictionary = resources.get("/XObject").unwrap().into();
        let second_content = second.get_page_content_data().unwrap();
        assert!(String::from_utf8_lossy(second_content.as_ref()).contains("0.00000000 -1.00000000"));
        for name in ["/B1", "/B2"] {
            let form: qpdf::QPdfStream = xobjects.get(name).unwrap().into();
            let data = form.get_data(StreamDecodeLevel::All).unwrap();
            assert!(!data.is_empty());
        }
    }

    #[test]
    fn assembles_rotated_pages_into_complete_four_up_sheets() {
        let directory = TempDir::new().unwrap();
        let source_path = directory.path().join("source.pdf");
        let output_path = directory.path().join("n-up.pdf");
        create_source(&source_path, 5);

        assemble_n_up_pdf(&source_path, &output_path, &[0, 1, 2, 3, 4], 4, false).unwrap();

        let output = QPdf::read(&output_path).unwrap();
        output.check_pdf().unwrap();
        assert_eq!(output.get_num_pages().unwrap(), 2);
        let first = output.get_page(0).unwrap();
        assert!(first.get("/Rotate").is_none());
        assert_eq!(
            page_box(&first.get("/MediaBox").unwrap()).unwrap(),
            [0.0, 0.0, 200.0, 300.0]
        );
        let content = first.get_page_content_data().unwrap();
        let content = String::from_utf8_lossy(content.as_ref());
        assert!(content.contains("/N2 Do"));
        assert!(content.contains("0.00000000 -1.00000000 1.00000000 0.00000000"));
    }

    #[test]
    fn normalizes_page_rotation_into_content_for_cups_n_up() {
        let directory = TempDir::new().unwrap();
        let source_path = directory.path().join("source.pdf");
        let output_path = directory.path().join("normalized.pdf");
        create_source(&source_path, 2);

        normalize_pdf_page_rotations(&source_path, &output_path).unwrap();

        let output = QPdf::read(&output_path).unwrap();
        output.check_pdf().unwrap();
        assert_eq!(output.get_num_pages().unwrap(), 2);
        let rotated = output.get_page(1).unwrap();
        assert!(rotated.get("/Rotate").is_none());
        assert_eq!(
            page_box(&rotated.get("/MediaBox").unwrap()).unwrap(),
            [0.0, 0.0, 300.0, 200.0]
        );
        let content = rotated.get_page_content_data().unwrap();
        assert!(String::from_utf8_lossy(content.as_ref())
            .contains("0.00000000 -1.00000000 1.00000000 0.00000000"));
    }

    #[test]
    fn rejects_out_of_bounds_source_pages() {
        let directory = TempDir::new().unwrap();
        let source_path = directory.path().join("source.pdf");
        let output_path = directory.path().join("booklet.pdf");
        create_source(&source_path, 1);

        assert!(assemble_booklet_pdf(&source_path, &output_path, &[[Some(1), None]]).is_err());
        assert!(!output_path.exists());
    }
}
