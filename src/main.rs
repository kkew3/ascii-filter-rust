use clap::Parser;

use ascii_filter::stdin_stdout_buffer_filter;

#[derive(Parser, Debug)]
struct App {
    /// Specify the buffer size, which default to 128.
    #[clap(short = 'b', value_name = "BUFFER_SIZE", default_value_t = 128)]
    buf_size: usize,
    /// To pass through a subset of ASCII characters only.
    #[clap(short = 'a', default_value_t = false)]
    ascii_only: bool,
}

fn main() {
    let app = App::parse();
    stdin_stdout_buffer_filter(app.buf_size, app.ascii_only)
}
