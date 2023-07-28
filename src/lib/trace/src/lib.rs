use colored_json::to_colored_json_auto;
use serde_json::Value as JsonValue;

use std::io::Write;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

pub fn trace_json(value: &JsonValue) {
    print!("{}", to_colored_json_auto(value).unwrap());
}

pub fn trace_json_ln(value: &JsonValue) {
    println!("{}", to_colored_json_auto(value).unwrap());
}

#[macro_export]
macro_rules! trace_json_ln {
    ($json:expr) => {{
        #[cfg(feature = "trace")]
        trace::trace_json_ln($json);
    }};
}

#[macro_export]
macro_rules! trace_json {
    ($json:expr) => {{
        #[cfg(feature = "trace")]
        trace::trace_json($json);
    }};
}

pub fn etraceln(
    filename: &str,
    module_path: &str,
    line: u32,
    col: u32,
    fn_name: &str,
    clr: Option<Color>,
    msg: &str,
) {
    etrace(filename, module_path, line, col, fn_name, clr, msg);
    println!();
}

pub fn traceln(
    filename: &str,
    module_path: &str,
    line: u32,
    col: u32,
    fn_name: &str,
    clr: Option<Color>,
    msg: &str,
) {
    trace(filename, module_path, line, col, fn_name, clr, msg);
    println!();
}

pub fn trace(
    _file: &str,
    module_path: &str,
    _line: u32,
    _col: u32, // unused
    fn_name: &str,
    clr: Option<Color>,
    msg: &str,
) {
    let bufwtr = BufferWriter::stderr(ColorChoice::Always);
    let mut buffer = bufwtr.buffer();

    let _ = buffer.reset();

    let (_, name) = module_path.rsplit_once(':').unwrap();
    // let prefix = format!("{}:", name);
    let _ = write!(&mut buffer, "{}:[", name);
    // let _ = write!(&mut buffer, "\t[");
    // let _ = write!(&mut buffer, " [");
    let _ = buffer.set_color(ColorSpec::new().set_intense(true).set_fg(Some(Color::Cyan)));
    let _ = write!(
        &mut buffer,
        "{}",    //:width$}",
        fn_name  // width = std::cmp::min(12, fn_name.len())
    );
    let _ = buffer.reset();
    let _ = write!(
        &mut buffer,
        // "{:width$}",
        "]",
        // width = 20 - std::cmp::min(20, fn_name.len() + name.len() + 4)
    );
    // let _ = write!(&mut buffer, "[{}]", buf2.);
    let _ = buffer.reset();

    if clr.is_some() {
        let _ = buffer.set_color(
            ColorSpec::new()
                .set_intense(true)
                .set_fg(Some(clr.unwrap())),
        );
    }
    let _ = write!(&mut buffer, " {}", msg);
    let _ = buffer.reset();

    // let (_, filename) = file.rsplit_once("/").unwrap();
    // let _ = write!(&mut buffer, " [{}:{}]", filename, line);
    let _ = bufwtr.print(&buffer);
}

pub fn etrace(
    file: &str,
    _module_path: &str,
    line: u32,
    _col: u32, // unused
    fn_name: &str,
    clr: Option<Color>,
    msg: &str,
) {
    let bufwtr = BufferWriter::stderr(ColorChoice::Always);
    let mut buffer = bufwtr.buffer();

    if clr.is_some() {
        let _ = buffer.set_color(
            ColorSpec::new()
                .set_intense(true)
                .set_fg(Some(clr.unwrap())),
        );
    }
    let _ = write!(&mut buffer, "{}", msg);

    let _ = buffer.reset();
    let _ = write!(&mut buffer, " (");

    let _ = buffer.reset();
    // let _ = write!(&mut buffer, "");
    let _ = buffer.set_color(
        ColorSpec::new()
            .set_intense(true)
            .set_fg(Some(Color::Magenta)),
    );
    let _ = write!(&mut buffer, "{}", fn_name);

    let _ = buffer.set_color(ColorSpec::new().set_intense(true).set_fg(Some(Color::Cyan)));
    let _ = write!(&mut buffer, ", {}", file);
    let _ = buffer.reset();
    let _ = write!(&mut buffer, ",");
    let _ = buffer.set_color(
        ColorSpec::new()
            .set_intense(false)
            .set_fg(Some(Color::Yellow)),
    );
    let _ = write!(&mut buffer, " line {}", line);
    let _ = buffer.reset();
    let _ = write!(&mut buffer, ")");

    let _ = buffer.reset();
    let _ = bufwtr.print(&buffer);
}

#[macro_export]
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        type_name_of(f)
            .rsplit("::")
            .find(|&part| part != "f" && part != "{{closure}}")
            .expect("Short function name")
    }};
}

#[macro_export]
macro_rules! trace_green_ln {
      ($($arg:tt)*) => {{
        #[cfg(feature = "trace")]
        trace::traceln(file!(), module_path!(), line!(), column!(),
         trace::function!(), Some(termcolor::Color::Green), format!("{}", std::format_args!($($arg)*)).as_str())
      }};
    }

#[macro_export]
macro_rules! trace_warn_ln {
  ($($arg:tt)*) => {{
    trace::etraceln(file!(), module_path!(), line!(), column!(),
     trace::function!(), Some(termcolor::Color::Yellow), format!("{}", std::format_args!($($arg)*)).as_str())
  }};
}

#[macro_export]
macro_rules! trace_err_ln {
  ($($arg:tt)*) => {{
    trace::etraceln(file!(), module_path!(), line!(), column!(),
      trace::function!(), Some(termcolor::Color::Red), format!("{}", std::format_args!($($arg)*)).as_str())
  }};
}

#[macro_export]
macro_rules! trace_info_ln {
  ($($arg:tt)*) => {{
    #[cfg(feature = "trace")]
    trace::traceln(file!(), module_path!(), line!(), column!(),
      trace::function!(), None, format!("{}", std::format_args!($($arg)*)).as_str())
  }};
}
