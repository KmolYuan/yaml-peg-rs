use crate::*;
use pom::{
    char_class::{alpha, alphanum, digit},
    parser::{call, empty, end, is_a, list, none_of, one_of, seq, sym, Parser},
};
use std::{io::Result, iter::FromIterator};

type Id = (String, String, usize);

macro_rules! to_string {
    ($e:expr) => {
        String::from_utf8_lossy($e).to_string()
    };
}

pub(crate) fn identifier<'a>() -> Parser<'a, u8, String> {
    let id = is_a(alpha) + (is_a(alphanum) | one_of(b"_-")).repeat(0..);
    id.collect().map(|s| to_string!(s)).name("identifier")
}

fn ty<'a>() -> Parser<'a, u8, String> {
    let t = sym(b'!') * sym(b'!').opt() * identifier();
    t.name("type assertion")
}

fn anchor<'a>() -> Parser<'a, u8, String> {
    let a = sym(b'&') * identifier();
    a.name("anchor defined")
}

fn anchor_use<'a>() -> Parser<'a, u8, String> {
    let a = sym(b'*') * identifier();
    a.name("anchor used")
}

fn comment<'a>() -> Parser<'a, u8, ()> {
    (sym(b'#') + none_of(b"\r\n").repeat(0..) + nw()).discard()
}

fn nw<'a>() -> Parser<'a, u8, ()> {
    one_of(b"\r\n").discard().name("newline")
}

fn indent<'a>() -> Parser<'a, u8, ()> {
    seq(b"  ").discard().name("indent")
}

fn ws<'a>() -> Parser<'a, u8, ()> {
    (sym(b' ').repeat(0..) - comment().opt())
        .discard()
        .name("white space")
}

fn ws_any<'a>() -> Parser<'a, u8, String> {
    let space = one_of(b" \t\r\n").repeat(0..) - comment().opt();
    space.collect().map(|_| " ".into()).name("any white space")
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
    let i = is_a(digit).repeat(1..).collect().map(|s| to_string!(s));
    i.name("integer")
}

fn number<'a>() -> Parser<'a, u8, String> {
    let frac = is_a(digit).repeat(0..) + sym(b'.') + (is_a(digit).repeat(1..));
    let exp = one_of(b"eE") + one_of(b"+-").opt() + is_a(digit).repeat(1..);
    let number = one_of(b"+-").opt() + frac + exp.opt();
    let n = number.collect().map(|s| to_string!(s));
    n.name("number")
}

fn inf_nan<'a>() -> Parser<'a, u8, String> {
    let inf = (one_of(b"+-") - sym(b'.') - one_of(b"iI") - one_of(b"nN") - one_of(b"fF"))
        .map(|s| to_string!(&[s]) + "inf");
    let nan = (sym(b'.') + one_of(b"nN") + one_of(b"aA") + one_of(b"nN")).map(|_| "NaN".into());
    inf.name("inf") | nan.name("NaN")
}

fn string_plain<'a>() -> Parser<'a, u8, String> {
    let string = none_of(b"\"\'?-#[]{},") + none_of(b":#[]{},").repeat(0..);
    string.collect().map(|s| to_string!(s)).name("plain string")
}

fn string_quoted<'a>() -> Parser<'a, u8, String> {
    let double = sym(b'"') * (none_of(b"\\\"") | escape()).repeat(0..) - sym(b'"');
    let single = sym(b'\'') * (none_of(b"\\\'") | escape()).repeat(0..) - sym(b'\'');
    let string = double.name("double quoted string") | single.name("single quoted string");
    string.convert(String::from_utf8).name("quoted string")
}

fn string_flow<'a>() -> Parser<'a, u8, String> {
    let s = string_quoted() | string_plain();
    s.name("flow string")
}

fn array_flow<'a>() -> Parser<'a, u8, Array> {
    let a = sym(b'[') * ws_any() * list(value(), sym(b',') - ws_any()) - ws_any() - sym(b']');
    a.name("flow array")
}

fn map_flow<'a>() -> Parser<'a, u8, Map> {
    let member = value() - sym(b':') + value();
    let m = sym(b'{') * ws_any() * list(member, sym(b',') - ws_any()) - ws_any() - sym(b'}');
    let m = m.map(|v| v.into_iter().collect());
    m.name("flow map")
}

fn array<'a, I: 'a>(id: I, level: usize) -> Parser<'a, u8, Node>
where
    I: Fn() -> Parser<'a, u8, Id>,
{
    let no_wrap = call(move || array(pos, level + 1)) | call(move || map(pos, level + 1));
    let wrap = call(move || array(prefix, level + 1)) | call(move || map(prefix, level + 1));
    let sub = value() | no_wrap | nw() * wrap;
    let item = sym(b'-') * ws() * sub;
    let item = list(item, nw() + indent().repeat(level));
    let a = id() + nw() * item - nw();
    let a = a.map(|((an, ty, pos), a)| node!(Yaml::from_iter(a), pos, an, ty));
    a.name("array")
}

fn map<'a, I: 'a>(id: I, level: usize) -> Parser<'a, u8, Node>
where
    I: Fn() -> Parser<'a, u8, Id>,
{
    let no_wrap = call(move || array(pos, level + 1)) | call(move || map(pos, level + 1));
    let wrap = call(move || array(prefix, level + 1)) | call(move || map(prefix, level + 1));
    let k1 = value();
    // TODO '?' key
    let sub = value() | no_wrap | nw() * wrap;
    let item = k1.name("key") - sym(b':') + sub;
    let item = list(item, nw() + indent().repeat(level));
    let m = id() + nw() * item - nw();
    let m = m.map(|((an, ty, pos), m)| node!(Yaml::from_iter(m), pos, an, ty));
    m.name("map")
}

fn value<'a>() -> Parser<'a, u8, Node> {
    let v = (prefix()
        + (sym(b'~').map(|_| Yaml::Null)
            | seq(b"null").map(|_| Yaml::Null)
            | seq(b"true").map(|_| Yaml::Bool(true))
            | seq(b"false").map(|_| Yaml::Bool(false))
            | integer().map(|num| Yaml::Int(num))
            | number().map(|num| Yaml::Float(num))
            | inf_nan().map(|num| Yaml::Float(num))
            | anchor_use().map(|a| Yaml::Anchor(a))
            | call(array_flow).map(|a| Yaml::Array(a))
            | call(map_flow).map(|m| Yaml::Map(m))
            | string_flow().map(|text| Yaml::Str(text))))
    .map(|((an, ty, pos), yaml)| node!(yaml, pos, an, ty))
        - ws_any();
    v.name("value")
}

fn pos<'a>() -> Parser<'a, u8, Id> {
    let p = ws() * empty().pos();
    p.map(|pos| ("".into(), "".into(), pos))
}

fn prefix<'a>() -> Parser<'a, u8, Id> {
    let p = ws() * anchor().opt().collect().map(|s| to_string!(s)) - ws()
        + ty().opt().collect().map(|s| to_string!(s))
        - ws()
        + empty().pos();
    p.map(|((an, ty), pos)| (an, ty, pos))
}

fn doc<'a>() -> Parser<'a, u8, Node> {
    let d = array(prefix, 0) | map(prefix, 0) | value();
    d.name("doc")
}

fn yaml<'a>() -> Parser<'a, u8, Array> {
    let total =
        seq(b"---\n").opt() * list(doc() - seq(b"...").opt(), seq(b"...").opt() + seq(b"---\n"));
    ws_any() * total - end()
}

/// Parse YAML document.
pub fn parse(doc: &str) -> Result<Array> {
    match yaml().parse(doc.as_bytes()) {
        Ok(e) => Ok(e),
        Err(e) => Err(error_indicator(e, doc)),
    }
}
