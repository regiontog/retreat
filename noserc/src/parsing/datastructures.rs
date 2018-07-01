use codegen;

pub type ScopeMutater = Box<Fn(&mut codegen::Scope) -> ()>;

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
    pub generic_over: Option<Box<Type>>,
}

impl Type {
    pub fn name_with_generics(&self) -> String {
        format!("{}{}", self.name, match &self.generic_over {
            Some(t) => format!("<{}>", t.name_with_generics()),
            None => "".to_string(),
        })
    }

    pub fn to_codegen_type(&self) -> codegen::Type {
        let mut t = codegen::Type::new(self.name.as_str());
        match &self.generic_over {
            Some(g) => {t.generic(g.to_codegen_type());},
            None => (),
        }

        t
    }
}
