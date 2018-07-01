use ::parsing::datastructures;
use ::parsing::common::{identifier, whitespace0, whitespace1, type_identifier};

named!(struct_field<&str,datastructures::Field>, do_parse!(
    name: identifier      >>
    whitespace0           >>
    tag!(":")             >>
    whitespace0           >>
    kind: type_identifier >>
    (datastructures::Field {name: name, kind: kind})
));

named!(struct_body<&str,Vec<datastructures::Field>>, delimited!(
    pair!(tag!("{"), whitespace0),
    separated_nonempty_list!(whitespace1, struct_field),
    pair!(whitespace0, tag!("}"))
));

named!(pub struct_type<&str,datastructures::ScopeMutater>, do_parse!(
    tag!("struct")      >>
    whitespace1         >>
    id: type_identifier >>
    whitespace0         >>
    fields: struct_body >>
    (Box::new(move |scope| {
        let strct = scope.new_struct(&id.name);

        match &id.generic_over {
            Some(t) => {strct.generic(&t.name_with_generics());},
            None => (),
        }
        
        for field in &fields {
            strct.field(field.name.as_str(), field.kind.to_codegen_type());
        }
    }))
));