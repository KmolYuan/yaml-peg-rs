use crate::*;
use pom::{
    char_class::{alpha, alphanum, digit},
    parser::{call, empty, end, is_a, list, none_of, one_of, seq, sym, Parser},
};
use std::io::Result;

type Id = (String, String, usize);

macro_rules! to_string {
    ($e:expr) => {
        String::from_utf8_lossy($e).to_string()
    };
}

pub(crate) fn identifier<'a>() -> Parser<'a, u8, String> {
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
    (sym(b'#') + none_of(b"\r\n").repeat(0..) + nw()).discard()
}

fn nw<'a>() -> Parser<'a, u8, ()> {
    one_of(b"\r\n").discard()
}

fn indent<'a>() -> Parser<'a, u8, &'a [u8]> {
    seq(b"  ")
}

fn ws<'a>() -> Parser<'a, u8, ()> {
    (sym(b' ').repeat(0..) - comment().opt()).discard()
}

fn ws_any<'a>() -> Parser<'a, u8, String> {
    let space = one_of(b" \t\r\n").repeat(0..) - comment().opt();
    space.collect().map(|_| " ".into())
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

fn string_plain<'a>() -> Parser<'a, u8, String> {
    let string = none_of(b"\"\'?-#[]{},") + none_of(b":#[]{},").repeat(0..);
    string.collect().map(|s| to_string!(s))
}

fn string_quoted<'a>() -> Parser<'a, u8, String> {
    let string = sym(b'"') * (none_of(b"\\\"") | escape()).repeat(0..) - sym(b'"');
    let string = string | sym(b'\'') * (none_of(b"\\\'") | escape()).repeat(0..) - sym(b'\'');
    string.convert(String::from_utf8)
}

fn string_flow<'a>() -> Parser<'a, u8, String> {
    string_quoted() | string_plain()
}

fn array_flow<'a>() -> Parser<'a, u8, Array> {
    sym(b'[') * ws_any() * list(value(), sym(b',') - ws_any()) - ws_any() - sym(b']')
}

fn map_flow<'a>() -> Parser<'a, u8, Map> {
    let member = empty().pos() + string_flow() - sym(b':') + value();
    let m = sym(b'{') * ws_any() * list(member, sym(b',') - ws_any()) - ws_any() - sym(b'}');
    m.map(|v| {
        v.iter()
            .map(|((pos, k), v)| (node!(k.into(), *pos), v.clone()))
            .into_iter()
            .collect()
    })
}

fn array<'a, I: 'a, L: 'a>(id: I, level: L) -> Parser<'a, u8, Node>
where
    I: Fn() -> Parser<'a, u8, Id>,
    L: Fn() -> Parser<'a, u8, &'a [u8]>,
{
    let no_wrap = call(|| array(pos, || (level() + indent()).collect()))
        | call(|| map(pos, || (level() + indent()).collect()));
    // FIXME
    let leading = (level() + indent().opt()).collect();
    let wrap = nw() * (call(|| array(prefix, || leading)) | call(|| map(prefix, || leading)));
    let sub = value() | no_wrap | wrap;
    let item = sym(b'-') * ws() * sub;
    let item = list(item, nw() + level());
    let a = id() + nw() * item - nw();
    a.map(|((an, ty, pos), a)| Node::new(Yaml::Array(a)).pos(pos).anchor(an).ty(ty))
}

// TODO
fn map<'a, I: 'a, L: 'a>(id: I, level: L) -> Parser<'a, u8, Node>
where
    I: Fn() -> Parser<'a, u8, Id>,
    L: Fn() -> Parser<'a, u8, &'a [u8]>,
{
    let item1 = level() * sym(b'?') * value() - sym(b':') + value();
    let item2 = level() * empty().pos() + string_flow() - sym(b':') + value();
    let item2 = item2.map(|((pos, k), v)| (node!(k.into(), pos), v));
    let item = item1 | item2;
    let m = prefix() + nw() * item.repeat(1..).map(|v| v.into_iter().collect::<Map>()) - nw();
    m.map(|((an, ty, pos), m)| Node::new(Yaml::Map(m)).pos(pos).anchor(an).ty(ty))
}

fn value<'a>() -> Parser<'a, u8, Node> {
    (prefix()
        + (sym(b'~').map(|_| Yaml::Null)
            | seq(b"null").map(|_| Yaml::Null)
            | seq(b"true").map(|_| Yaml::Bool(true))
            | seq(b"false").map(|_| Yaml::Bool(false))
            | integer().map(|num| Yaml::Int(num))
            | number().map(|num| Yaml::Float(num))
            | inf_nan().map(|num| Yaml::Float(num))
            | anchor_use().map(|a| Yaml::Anchor(a))
            | string_flow().map(|text| Yaml::Str(text))
            | call(array_flow).map(|a| Yaml::Array(a))
            | call(map_flow).map(|m| Yaml::Map(m))))
    .map(|((a, ty, pos), yaml)| Node::new(yaml).pos(pos).ty(ty).anchor(a))
        - ws_any()
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

fn dummy<'a>() -> Parser<'a, u8, &'a [u8]> {
    sym(b' ').repeat(0).collect()
}

fn documentation<'a>() -> Parser<'a, u8, Node> {
    seq(b"---").opt() * (call(|| array(prefix, dummy)) | call(|| map(prefix, dummy)) | value())
        - seq(b"...").opt()
}

fn yaml<'a>() -> Parser<'a, u8, Array> {
    documentation().repeat(1..) - end()
}

/// Parse YAML document.
pub fn parse_yaml(doc: &str) -> Result<Array> {
    match yaml().parse(doc.as_bytes()) {
        Ok(e) => Ok(e),
        Err(e) => Err(error_indicator(e, doc)),
    }
}
