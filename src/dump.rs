use prettytable::{format, row, Cell, Row, Table};

use crate::eval::ProcMetrics;

/// Print a table view of process metrics with their descriptions.
pub fn print_proc_metrics<'a>(
    proc_metrics: impl Iterator<Item = ProcMetrics>,
    descriptions: impl Iterator<Item = &'a str>,
) {
    let mut table = Table::new();

    let format = format::FormatBuilder::new()
        .column_separator('|')
        .borders('|')
        .padding(1, 1)
        .build();
    table.set_format(format);

    table.add_row(row![
        "Process",
        "CPU %",
        "CPU % stddev",
        "total mean",
        "total stddev",
        "utime mean",
        "stime mean",
    ]);

    for (metric, description) in proc_metrics.zip(descriptions) {
        let mut desc = Cell::new(description);
        desc.align(format::Alignment::LEFT);
        table.add_row(Row::new(vec![
            desc,
            right_aligned_cell(&format!("{:.2}", metric.cpu_usage.mean)),
            right_aligned_cell(&format!("{:.2}", metric.cpu_usage.stddev)),
            right_aligned_cell(&format!("{:.2}", metric.total.mean)),
            right_aligned_cell(&format!("{:.2}", metric.total.stddev)),
            right_aligned_cell(&format!("{:.2}", metric.user.mean)),
            right_aligned_cell(&format!("{:.2}", metric.system.mean)),
        ]));
    }

    table.printstd();
}

/// Clear the last n lines using ANSI escape sequences.
pub fn clear_n_lines(n: usize) {
    print!("\x1b[{}A", n);
}

fn right_aligned_cell(msg: &str) -> Cell {
    let mut cell = Cell::new(msg);
    cell.align(format::Alignment::RIGHT);
    cell
}
