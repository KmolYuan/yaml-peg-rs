use crate::*;
use pom::{
    char_class::{alpha, alphanum, digit},
    parser::{call, empty, end, is_a, list, none_of, one_of, seq, sym, Parser},
    set::Set,
};
use std::io::Result;

fn identifier<'a>() -> Parser<'a, u8, String> {
    let id = is_a(alpha) + is_a(|c| alphanum(c) || c == b'_').repeat(0..);
    id.collect().map(|s| s.to_str().into())
}

fn ty<'a>() -> Parser<'a, u8, String> {
    sym(b'!') * sym(b'!').opt() * identifier()
}

fn comment<'a>() -> Parser<'a, u8, ()> {
    (sym(b'#') + none_of(b"\n").repeat(0..)).discard()
}

fn ws<'a>() -> Parser<'a, u8, ()> {
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

fn number<'a>() -> Parser<'a, u8, String> {
    let integer = is_a(digit).repeat(1..);
    let frac = sym(b'.').opt() + (is_a(digit).repeat(0..));
    let exp = one_of(b"eE") + one_of(b"+-").opt() + is_a(digit).repeat(1..);
    let number = sym(b'-').opt() + integer + frac + exp.opt();
    number.collect().map(|s| s.to_str().into())
}

fn inf<'a>() -> Parser<'a, u8, String> {
    (sym(b'.') + one_of(b"iI") + one_of(b"nN") + one_of(b"fF")).map(|_| "inf".into())
}

fn nan<'a>() -> Parser<'a, u8, String> {
    (sym(b'.') + one_of(b"nN") + one_of(b"aA") + one_of(b"nN")).map(|_| "NaN".into())
}

fn string_flow<'a>() -> Parser<'a, u8, String> {
    let string = sym(b'"') * (none_of(b"\\\"") | escape()).repeat(0..) - sym(b'"');
    string.convert(String::from_utf8)
}

fn array_flow<'a>() -> Parser<'a, u8, Array> {
    sym(b'[') * ws() * list(value(), sym(b',') * ws()) - ws() - sym(b']')
}

fn map_flow<'a>() -> Parser<'a, u8, Map> {
    let member = value() - sym(b':') + value();
    let members = list(member, sym(b',') * ws());
    let obj = sym(b'{') * ws() * members - ws() - sym(b'}');
    obj.map(|members| members.into_iter().collect())
}

fn value<'a>() -> Parser<'a, u8, Node> {
    (ws() * ty().opt() - ws()
        + empty().pos()
        + (sym(b'~').map(|_| Yaml::Null)
            | seq(b"null").map(|_| Yaml::Null)
            | seq(b"true").map(|_| Yaml::Bool(true))
            | seq(b"false").map(|_| Yaml::Bool(false))
            | number().map(|num| Yaml::Str(num))
            | inf().map(|num| Yaml::Str(num))
            | nan().map(|num| Yaml::Str(num))
            | string_flow().map(|text| Yaml::Str(text))
            | call(array_flow).map(|arr| Yaml::Array(arr))
            | call(map_flow).map(|obj| Yaml::Map(obj))))
    .map(|((ty, pos), yaml)| Node::new(yaml).pos(pos).ty(ty))
        - ws()
}

fn yaml<'a>() -> Parser<'a, u8, Array> {
    (seq(b"---").opt() * value() - seq(b"...").opt()).repeat(1..) - end()
}

pub fn parse_yaml(doc: &str) -> Result<Array> {
    match yaml().parse(doc.as_bytes()) {
        Ok(e) => Ok(e),
        Err(e) => Err(error_indicator(e, doc)),
    }
}
