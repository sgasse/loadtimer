use prettytable::{row, Table};

use crate::eval::ProcMetrics;

/// Print a table view of process metrics with their descriptions.
pub fn print_proc_metrics(
    proc_metrics: impl Iterator<Item = ProcMetrics>,
    descriptions: impl Iterator<Item = String>,
) {
    let mut table = Table::new();

    use prettytable::format;

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
        table.add_row(row![
            r =>
            description,
            format!("{:.1}", metric.cpu_usage.mean),
            format!("{:.2}", metric.cpu_usage.stddev),
            format!("{:.1}", metric.total.mean),
            format!("{:.1}", metric.total.stddev),
            format!("{:.1}", metric.user.mean),
            format!("{:.1}", metric.system.mean),
        ]);
    }

    table.printstd();
}

/// Clear the last n lines using ANSI escape sequences.
pub fn clear_n_lines(n: usize) {
    print!("\x1b[{}A", n);
}
