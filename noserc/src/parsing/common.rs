use nom;

use parsing::datastructures;

pub fn eof(input: &str) -> nom::IResult<&str, &str> {
    if input.len() == 0 {
        Ok((&input[0..], &input[0..0]))
    } else {
        Err(nom::Err::Error(nom::Context::Code(
            input,
            nom::ErrorKind::Eof,
        )))
    }
}

pub fn whitespace(input: &str) -> nom::IResult<&str, &str> {
    const SPACES: &str = ", \t\n\r";

    if input.len() < 1 {
        return Err(nom::Err::Incomplete(nom::Needed::Size(1)));
    }

    match SPACES.find(|c| input.starts_with(c)) {
        Some(_) => Ok((&input[1..], &input[0..1])),
        None => Err(nom::Err::Error(nom::Context::Code(
            input,
            nom::ErrorKind::AlphaNumeric,
        ))),
    }
}

named!(pub whitespace1<&str,String>, map!(many1!(whitespace), |vec| vec.join("")));
named!(pub whitespace0<&str,String>, map!(many0!(whitespace), |vec| vec.join("")));
named!(pub whitespace0_complete<&str,String>, map!(many0!(complete!(whitespace)), |vec| vec.join("")));

pub fn alphanumeric(input: &str) -> nom::IResult<&str, &str> {
    const ABC: &str = "1234567890abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    if input.len() < 1 {
        return Err(nom::Err::Incomplete(nom::Needed::Size(1)));
    }

    match ABC.find(|c| input.starts_with(c)) {
        Some(_) => Ok((&input[1..], &input[0..1])),
        None => Err(nom::Err::Error(nom::Context::Code(
            input,
            nom::ErrorKind::AlphaNumeric,
        ))),
    }
}

named!(underscore<&str,&str>, tag!("_"));
named!(alphanumeric_<&str,&str>, alt!(underscore | alphanumeric));

named!(pub identifier<&str,String>, map!(
    alt!(
        tuple!(alphanumeric, many0!(alphanumeric_)) |
        tuple!(underscore, many1!(alphanumeric_))
    ),
    |tuple| {
        let mut id = String::with_capacity(tuple.1.len() + 1);

        for s in vec![tuple.0].into_iter().chain(tuple.1) {
            id.push_str(s);
        }

        id
    }
));

named!(lifetime<&str,datastructures::Type>, map!(
    preceded!(tag!("'"), identifier),
    |id| datastructures::Type {
        name: format!("'{}", id),
        generic_over: vec![],
    }
));

named!(generic<&str,Vec<datastructures::Type>>, delimited!(
    pair!(tag!("<"), whitespace0),
    separated_nonempty_list!(whitespace1, alt!(type_path | lifetime)),
    pair!(whitespace0, tag!(">"))
));

named!(pub type_identifier<&str,datastructures::Type>, do_parse!(
    name: identifier        >>
    generics: opt!(generic) >>
    (datastructures::Type { name: name, generic_over: match generics {
        Some(kinds) => kinds,
        None => vec![]
    }})
));

named!(pub type_path<&str,datastructures::Type>, do_parse!(
    abs_indicator: opt!(tag!("::"))             >>
    path: many0!(pair!(identifier, tag!("::"))) >>
    kind: type_identifier                       >>
    (datastructures::Type {
        name: format!(
            "{}{}{}",
             abs_indicator.unwrap_or(""),
             path.into_iter().map(|pair| pair.0 + pair.1).collect::<Vec<_>>().join(""),
             kind.name
        ),
        generic_over: kind.generic_over,
    })
));
