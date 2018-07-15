use codegen;

use ::parsing::datastructures;
use ::parsing::common::{identifier, whitespace0, whitespace1, type_identifier};

named!(union_subtype<&str,datastructures::Type>, delimited!(
    pair!(tag!("("), whitespace0),
    type_identifier,
    pair!(whitespace0, tag!(")"))
));

named!(union_field<&str,datastructures::EnumVariant>, do_parse!(
    name: identifier             >>
    subtype: opt!(union_subtype) >>
    (datastructures::EnumVariant {name: name, subtype: subtype})
));

named!(union_body<&str,Vec<datastructures::EnumVariant>>, delimited!(
    pair!(tag!("{"), whitespace0),
    separated_nonempty_list!(whitespace1, union_field),
    pair!(whitespace0, tag!("}"))
));

named!(pub union_type<&str,datastructures::ScopeMutater>, do_parse!(
    tag!("union")       >>
    whitespace1         >>
    id: type_identifier >>
    whitespace0         >>
    fields: union_body  >>
    (Box::new(move |scope| {
        let enm = scope.new_enum(&id.name);

        match &id.generic_over {
            Some(t) => {enm.generic(&t.name_with_generics());},
            None => (),
        }

        for field in fields.iter() {
            let mut var = codegen::Variant::new(field.name.as_str());
            match &field.subtype {
                Some(t) => { var.tuple(&t.name_with_generics()); },
                None => (),
            }

            enm.push_variant(var);
        }
    }))
));