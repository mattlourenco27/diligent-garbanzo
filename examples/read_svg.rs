use std::env;
use std::ffi::OsString;
use std::path::Path;

use drawsvg::svg;

const USAGE: &'static str = "Usage: read_svg <svg file>";

fn main() {
    let args: Vec<OsString> = env::args_os().collect();
    if args.len() < 2 {
        println!("{}", USAGE);
        return;
    }

    let svg_path = Path::new(&args[1]);
    match svg::read_from_file(svg_path) {
        Err(err) => println!("{}", err),
        _ => (),
    };
}
