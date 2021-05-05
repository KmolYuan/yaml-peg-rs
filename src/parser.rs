use crate::*;
use pom::{
    char_class::{alpha, alphanum, digit},
    parser::{call, empty, end, is_a, list, none_of, one_of, seq, sym, Parser},
};
use std::io::Result;

macro_rules! to_string {
    ($e:expr) => {
        String::from_utf8_lossy($e).to_string()
    };
}

fn identifier<'a>() -> Parser<'a, u8, String> {
    let id = is_a(alpha) + (is_a(alphanum) | one_of(b"_-")).repeat(0..);
    id.collect().map(|s| to_string!(s))
}

fn ty<'a>() -> Parser<'a, u8, String> {
    sym(b'!') * sym(b'!').opt() * identifier()
}

fn anchor<'a>() -> Parser<'a, u8, String> {
    sym(b'&') * identifier()
}

fn anchor_use<'a>() -> Parser<'a, u8, String> {
    sym(b'*') * identifier()
}

fn comment<'a>() -> Parser<'a, u8, ()> {
    (sym(b'#') + none_of(b"\n").repeat(0..)).discard()
}

fn ws<'a>() -> Parser<'a, u8, ()> {
    (one_of(b" ").repeat(0..) - comment().opt()).discard()
}

fn ws_any<'a>() -> Parser<'a, u8, ()> {
    (one_of(b" \t\r\n").repeat(0..) - comment().opt()).discard()
}

fn escape<'a>() -> Parser<'a, u8, u8> {
    sym(b'\\')
        * (sym(b'\\')
            | sym(b'/')
            | sym(b'"')
            | sym(b'b').map(|_| b'\x08')
            | sym(b'f').map(|_| b'\x0C')
            | sym(b'n').map(|_| b'\n')
            | sym(b'r').map(|_| b'\r')
            | sym(b't').map(|_| b'\t'))
}

fn integer<'a>() -> Parser<'a, u8, String> {
    is_a(digit).repeat(1..).collect().map(|s| to_string!(s))
}

fn number<'a>() -> Parser<'a, u8, String> {
    let frac = is_a(digit).repeat(0..) + sym(b'.') + (is_a(digit).repeat(1..));
    let exp = one_of(b"eE") + one_of(b"+-").opt() + is_a(digit).repeat(1..);
    let number = one_of(b"+-").opt() + frac + exp.opt();
    number.collect().map(|s| to_string!(s))
}

fn inf_nan<'a>() -> Parser<'a, u8, String> {
    (one_of(b"+-") - sym(b'.') - one_of(b"iI") - one_of(b"nN") - one_of(b"fF"))
        .map(|s| to_string!(&[s]) + "inf")
        | (sym(b'.') + one_of(b"nN") + one_of(b"aA") + one_of(b"nN")).map(|_| "NaN".into())
}

fn string_literal<'a>() -> Parser<'a, u8, String> {
    let string = none_of(b"[]{}&*:,\"").repeat(1..);
    string.convert(String::from_utf8)
}

fn string_quoted<'a>() -> Parser<'a, u8, String> {
    let string = sym(b'"') * (none_of(b"\\\"") | escape()).repeat(0..) - sym(b'"');
    string.convert(String::from_utf8)
}

fn string_flow<'a>() -> Parser<'a, u8, String> {
    string_quoted() | string_literal()
}

fn array_flow<'a>() -> Parser<'a, u8, Array> {
    sym(b'[') * ws_any() * list(value(), sym(b',') * ws_any()) - ws_any() - sym(b']')
}

fn map_flow<'a>() -> Parser<'a, u8, Map> {
    let member = empty().pos() + string_flow() - sym(b':') + value();
    let obj = sym(b'{') * ws_any() * list(member, sym(b',') * ws_any()) - ws_any() - sym(b'}');
    obj.map(|members| {
        members
            .iter()
            .map(|((pos, k), v)| (Node::new(Yaml::Str(k.clone())).pos(*pos), v.clone()))
            .into_iter()
            .collect()
    })
}

fn value<'a>() -> Parser<'a, u8, Node> {
    (ws() * anchor().opt() - ws() + ty().opt() - ws()
        + empty().pos()
        + (sym(b'~').map(|_| Yaml::Null)
            | seq(b"null").map(|_| Yaml::Null)
            | seq(b"true").map(|_| Yaml::Bool(true))
            | seq(b"false").map(|_| Yaml::Bool(false))
            | integer().map(|num| Yaml::Int(num))
            | number().map(|num| Yaml::Float(num))
            | inf_nan().map(|num| Yaml::Float(num))
            | anchor_use().map(|a| Yaml::Anchor(a))
            | string_flow().map(|text| Yaml::Str(text))
            | call(array_flow).map(|arr| Yaml::Array(arr))
            | call(map_flow).map(|obj| Yaml::Map(obj))))
    .map(|(((a, ty), pos), yaml)| Node::new(yaml).pos(pos).ty(ty).anchor(a))
        - ws()
}

fn yaml<'a>() -> Parser<'a, u8, Array> {
    (seq(b"---").opt() * value() - seq(b"...").opt()).repeat(1..) - end()
}

/// Parse YAML document.
pub fn parse_yaml(doc: &str) -> Result<Array> {
    match yaml().parse(doc.as_bytes()) {
        Ok(e) => Ok(e),
        Err(e) => Err(error_indicator(e, doc)),
    }
}
