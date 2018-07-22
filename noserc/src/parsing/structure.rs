use parsing::common::{identifier, type_identifier, type_path, whitespace0, whitespace1};
use parsing::datastructures;

named!(struct_field<&str,datastructures::Field>, do_parse!(
    name: identifier      >>
    whitespace0           >>
    tag!(":")             >>
    whitespace0           >>
    kind: type_path       >>
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
    (Box::new(move |options, scope| {
        let mut strct = ::codegen::Struct::new(&id.name);
        let mut build = ::codegen::Impl::new(strct.ty());
        let mut impel = ::codegen::Impl::new(strct.ty());
        let mut build_trait = ::codegen::Type::new(&[options.noser_path, ::types::BUILD].join(""));
        let mut build_fn = ::codegen::Function::new("build");
        let mut with_fields = ::codegen::Function::new("with_fields");

        for t in &id.generic_over {
            strct.generic(&t.name_with_generics());
            impel.generic(&t.name_with_generics());
            impel.target_generic(&t.name_with_generics());
            build.target_generic(&t.name_with_generics());
        }

        for field in &fields {
            strct.field(field.name.as_str(), field.kind.to_codegen_type());
        }

        with_fields.ret(format!("{}::Record<'arena, {}>", options.noser_path, id.name_with_generics()));

        for (i, field) in fields.iter().enumerate() {
            with_fields.arg(field.name.as_str(), format!("F{}", i));
        }

        if !id.has_arena_lifetime_generic() {
            with_fields.generic("'arena");
        }

        for i in 0..fields.len() {
            with_fields.generic(format!("F{}", i).as_str());
        }

        for (i, field) in fields.iter().enumerate() {
            with_fields.bound(
                format!("F{}", i).as_str(),
                format!("'arena + {0}::traits::DynamicSize + {0}::traits::WithArena<'arena, {1}>", options.noser_path, field.kind.name_with_generics())
            );
        }

        with_fields.line("let mut size = 0;");

        for field in &fields {
            with_fields.line(format!("size += {}.dsize();", field.name.as_str()));
        }

        with_fields.line(format!("{}::Record::new(size, |arena: &'arena mut [u8]| {{", options.noser_path));

        for field in &fields {
            with_fields.line(format!(
                "    let ({0}_a, arena) = arena.split_at_mut({0}.dsize());",
                field.name.as_str()
            ));
        }

        with_fields.line(format!("    {} {{", id.name));
        for field in &fields {
            with_fields.line(format!("            {0}: {0}.with_arena({0}_a),", field.name));
        }

        with_fields.line("    }");
        with_fields.line("})");

        build.generic("'arena");
        build_trait.generic("'arena");

        for generic in &id.generic_over {
            if generic.name != "'arena" {
                build.generic(&generic.name_with_generics());
                build.bound(&generic.name_with_generics(), &build_trait);
            }
        }

        build.impl_trait(build_trait);

        build_fn.arg("arena", "&'arena mut [u8]");
        build_fn.ret("(&'arena mut [u8], Self)");

        for field in &fields {
            build_fn.line(format!("let (arena, {}) = {}::build(arena);", field.name, field.kind.name));
        }

        build_fn.line(format!("(arena, {} {{", &id.name));
        for field in &fields {
            build_fn.line(format!("    {}: {},", field.name, field.name));
        }

        build_fn.line("})");

        impel.push_fn(with_fields);
        build.push_fn(build_fn);

        scope.push_struct(strct);
        scope.push_impl(impel);
        scope.push_impl(build);
    }))
));
