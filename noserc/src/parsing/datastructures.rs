use codegen;

pub type ScopeMutater = Box<Fn(&::CompilerOptions, &mut codegen::Scope) -> ()>;

pub struct Field {
    pub name: String,
    pub kind: Type,
}

pub struct EnumVariant {
    pub name: String,
    pub subtype: Option<Type>,
}

pub struct Type {
    pub name: String,
    pub generic_over: Vec<Type>,
}

impl Type {
    pub fn name_with_generics(&self) -> String {
        format!(
            "{}{}",
            self.name,
            match &self.generic_over.len() {
                0 => "".to_string(),
                _ => format!(
                    "<{}>",
                    self.generic_over
                        .iter()
                        .map(|t| t.name_with_generics())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            }
        )
    }

    pub fn to_codegen_type(&self) -> codegen::Type {
        let mut t = codegen::Type::new(self.name.as_str());

        for g in &self.generic_over {
            t.generic(g.to_codegen_type());
        }

        t
    }

    pub fn has_arena_lifetime_generic(&self) -> bool {
        self.generic_over
            .iter()
            .any(|t| t.name_with_generics() == "'arena")
    }
}
