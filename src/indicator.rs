use alloc::{format, string::String};

/// Indicate the position of the documentation.
/// This function will show the line number and column number of the position.
///
/// ```
/// use yaml_peg::indicated_msg;
///
/// let doc = indicated_msg("{\"a\": \n[\"b\", \"c\", \"d\"]}", 13);
/// assert_eq!(doc, "2:7\n[\"b\", \"c\", \"d\"]}\n      ^")
/// ```
///
/// If print the string, it would be like:
///
/// ```bash
/// 2:7
/// ["b", "c", "d"]}
///       ^
/// ```
///
/// This may be what you need if you went to indicate an error on the invalid data.
pub fn indicated_msg(doc: &str, mut pos: u64) -> String {
    for (line, str_line) in doc.split('\n').enumerate() {
        let full_line = str_line.len() as u64 + 1;
        if full_line > pos {
            let column = pos as usize;
            return format!(
                "{}:{}\n{}\n{}^",
                line + 1,
                column + 1,
                str_line,
                " ".repeat(column)
            );
        } else {
            pos -= full_line;
        }
    }
    unreachable!()
}

/// Same as [`indicated_msg`], but join the path before message.
///
/// ```
/// use yaml_peg::indicated_msg_file;
///
/// let doc = indicated_msg_file("my/file.yaml", "{\"a\": \n[\"b\", \"c\", \"d\"]}", 13);
/// assert_eq!(doc, "my/file.yaml:2:7\n[\"b\", \"c\", \"d\"]}\n      ^")
/// ```
///
/// If print the string, it would be like:
///
/// ```bash
/// my/file.yaml:2:7
/// ["b", "c", "d"]}
///       ^
/// ```
pub fn indicated_msg_file(path: &str, doc: &str, pos: u64) -> String {
    format!("{}:{}", path, indicated_msg(doc, pos))
}
