use pom::Error;

/// Indicate the position of the documentation.
/// This function will show the line number and column number of the position.
///
/// ```
/// use yaml_pom::indicated_msg;
/// let doc = indicated_msg("{\"a\": \n[\"b\", \"c\", \"d\"]}", 12);
/// assert_eq!(doc, "(2:7)\n[\"b\", \"c\", \"d\"]}\n~~~~~~^")
/// ```
///
/// If print the string, it would be like:
///
/// ```bash
/// (2:7)
/// ["b", "c", "d"]}
/// ~~~~~~^
/// ```
///
/// This may be what you need if you went to indicate an error on invalid data.
pub fn indicated_msg(doc: &str, mut pos: usize) -> String {
    let mut show_line = String::new();
    for (line, str_line) in doc.split('\n').enumerate() {
        let full_line = str_line.len();
        if full_line > pos {
            let column = pos;
            show_line = format!(
                "({}:{})\n{}\n{}^",
                line + 1,
                column + 1,
                str_line,
                "~".repeat(column)
            );
            break;
        } else {
            pos -= full_line;
        }
    }
    show_line
}

pub(crate) fn error_indicator(e: Error, doc: &str) -> std::io::Error {
    match e {
        Error::Incomplete => err!("incomplete"),
        Error::Mismatch { position, message } => {
            err!(format!(
                "mismatch error: {}\n\n{}",
                indicated_msg(doc, position),
                message
            ))
        }
        Error::Conversion { position, message } => {
            err!(format!(
                "conversion error: {}\n\n{}",
                indicated_msg(doc, position),
                message
            ))
        }
        Error::Expect {
            position, message, ..
        } => {
            err!(format!(
                "expect error: {}\n\n{}",
                indicated_msg(doc, position),
                message
            ))
        }
        Error::Custom {
            position, message, ..
        } => {
            err!(format!(
                "custom error: {}\n\n{}",
                indicated_msg(doc, position),
                message
            ))
        }
    }
}
