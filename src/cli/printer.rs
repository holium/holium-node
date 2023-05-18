// This file contains the printer function for the CLI
// over time this will be expanded to include more functionality

use term_table::row::Row;
use term_table::table_cell::{Alignment, TableCell};
use term_table::Table;

pub fn print_to_cli<T: std::fmt::Display>(text: T) {
    println!("{}", text);
}

#[allow(dead_code)]
fn print_table(data: Vec<Vec<String>>) {
    let mut table = Table::new();
    table.max_column_width = 30;
    table.style = term_table::TableStyle::thin();

    // Add the header row.
    let mut header_row = Vec::new();
    for i in 0..data[0].len() {
        header_row.push(TableCell::new(format!("Column {}", i + 1)));
    }
    table.add_row(Row::new(header_row));

    // Add the data rows.
    for row_data in data {
        let mut row = Vec::new();
        for cell_data in row_data {
            row.push(TableCell::new_with_alignment(
                &cell_data,
                1,
                Alignment::Left,
            ));
        }
        table.add_row(Row::new(row));
    }

    println!("{}", table.render());
}
