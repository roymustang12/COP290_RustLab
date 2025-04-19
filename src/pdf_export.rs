use crate::cell_extension::SpreadsheetExtension;
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use chrono::Local;

fn get_col_label(col: i32) -> String {
    let mut col = col + 1; // 1-based index
    let mut label = String::new();
    while col > 0 {
        col -= 1;
        label.push((b'A' + (col % 26) as u8) as char);
        col /= 26;
    }
    label.chars().rev().collect()
}

pub fn export_to_pdf(sheet: &SpreadsheetExtension, filename: &str, title: Option<&str>, start_row: i32, start_col: i32, 
                      end_row: i32, end_col: i32) -> Result<(), String> {
    // Create a document with printpdf 0.8.2 API
    let document_title = title.unwrap_or("Spreadsheet Export");
    let (doc, page_index, layer_index) = PdfDocument::new(
        document_title,
        Mm(210.0),  // A4 width
        Mm(297.0),  // A4 height
        "Layer 1",
    );
    
    let current_layer = doc.get_page(page_index).get_layer(layer_index);
    
    // Set up fonts
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(|e| e.to_string())?;
    
    // Calculate dimensions
    let margin = 20.0;
    let cell_width = 40.0;
    let cell_height = 20.0;
    let header_height = 30.0;
    
    // Draw header
    let header_text = format!("{} - Generated on {}", 
                              document_title,
                              Local::now().format("%Y-%m-%d %H:%M"));
    current_layer.use_text(header_text, 12.0, Mm(margin), Mm(297.0 - margin), &font);
    
    // Draw table headers (column labels)
    for (idx, col) in (start_col..=end_col).enumerate() {
        let col_label = get_col_label(col);
        let x_pos = margin + (idx as f64 * cell_width);
        current_layer.use_text(col_label, 10.0, Mm(x_pos), Mm(297.0 - margin - header_height), &font);
    }
    
    // Draw table data
    for (r_idx, row) in (start_row..=end_row).enumerate() {
        // Draw row number
        let y_pos = 297.0 - margin - header_height - ((r_idx as f64 + 1.0) * cell_height);
        current_layer.use_text(format!("{}", row + 1), 9.0, Mm(margin - 15.0), Mm(y_pos), &font);
        
        // Draw cell values
        for (c_idx, col) in (start_col..=end_col).enumerate() {
            if row >= 0 && row < sheet.rows && col >= 0 && col < sheet.columns {
                let cell = &sheet.all_cells[row as usize][col as usize];
                let x_pos = margin + (c_idx as f64 * cell_width);
                
                let cell_text = if cell.is_error {
                    "ERROR".to_string()
                } else {
                    // Format numeric values nicely
                    if cell.value as f64 != cell.value as i32 as f64 {
                        format!("{:.2}", cell.value as f64) 
                    } else {
                        format!("{}", cell.value)
                    }
                };
                
                current_layer.use_text(cell_text, 9.0, Mm(x_pos), Mm(y_pos), &font);
            }
        }
    }
    
    // Draw grid lines (optional but makes it look more like a spreadsheet)
    let table_width = cell_width * ((end_col - start_col + 1) as f64);
    let table_height = cell_height * ((end_row - start_row + 1) as f64);
    
    // Horizontal lines
    for i in 0..=(end_row - start_row + 1) {
        let y = 297.0 - margin - header_height - (i as f64 * cell_height);
        let line = Line {
            points: vec![(Point::new(Mm(margin), Mm(y)), false), 
                         (Point::new(Mm(margin + table_width), Mm(y)), false)],
            is_closed: false,
            has_fill: false,
            has_stroke: true,
            is_clipping_path: false,
        };
        current_layer.add_shape(line);
    }
    
    // Vertical lines
    for i in 0..=(end_col - start_col + 1) {
        let x = margin + (i as f64 * cell_width);
        let line = Line {
            points: vec![(Point::new(Mm(x), Mm(297.0 - margin - header_height)), false), 
                         (Point::new(Mm(x), Mm(297.0 - margin - header_height - table_height)), false)],
            is_closed: false,
            has_fill: false,
            has_stroke: true,
            is_clipping_path: false,
        };
        current_layer.add_shape(line);
    }
    
    // Save the PDF
    doc.save(&mut BufWriter::new(File::create(filename).map_err(|e| e.to_string())?))
        .map_err(|e| e.to_string())?;
    
    Ok(())
}