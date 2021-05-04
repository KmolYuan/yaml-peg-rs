use pom::Error;

pub fn indicated_msg(doc: &str, mut pos: usize) -> String {
    let mut show_line = String::new();
    for (line, str_line) in doc.split('\n').enumerate() {
        let full_line = str_line.len();
        if full_line > pos {
            let column = pos;
            show_line = format!(
                "({}:{})\n{}\n{}^",
                line,
                column,
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

pub fn error_indicator(e: Error, doc: &str) -> std::io::Error {
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
