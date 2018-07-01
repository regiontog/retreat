#[macro_use]
extern crate nom;
extern crate codegen;

mod parsing;

use std::env;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::fs;

use ::parsing::common::{whitespace0, whitespace0_complete, eof};
use ::parsing::structure::{struct_type};
use ::parsing::union::{union_type};

named!(noser_parser<&str,String>, map!(
    delimited!(
        whitespace0,
        dbg_dmp!(separated_nonempty_list!(whitespace0_complete, complete!(alt!(struct_type | union_type)))),
        pair!(whitespace0_complete, eof)
    ), |mutators| {
        let mut scope = codegen::Scope::new();

        for mutator in mutators {
            mutator(&mut scope);
        }
        
        scope.to_string()
    }
));

pub struct NoserCompiler<'a> { 
    prefix: &'a str,
    base: Option<&'a str>,
    files: Vec<&'a str>,
}

impl<'a> NoserCompiler<'a> {
    pub fn new() -> NoserCompiler<'a> {
        NoserCompiler {
            prefix: "",
            base: None,
            files: Vec::new(),
        }
    }

    pub fn base(mut self, base: &'a str) -> Self {
        self.base = Some(base);
        self
    }

    pub fn remove_prefix(mut self, prefix: &'a str) -> Self {
        self.prefix = prefix;
        self
    }

    pub fn file(mut self, prefix: &'a str) -> Self {
        self.files.push(prefix);
        self
    }

    pub fn run(self) -> Result<(), NoserError<String>> {
        let prefix_path = Path::new(self.prefix);
        let base_str = match self.base {
            None => env::var("OUT_DIR")?,
            Some(base) => base.to_string(),
        };

        let base = Path::new(&base_str);

        let mut buffer = String::with_capacity(1024);

        for file_handle in self.files.iter() {
            let mut file = File::open(file_handle)?;
            file.read_to_string(&mut buffer)?;

            let (_, result) = noser_parser(&buffer)?;
            buffer.truncate(0);

            let mut out_path = base.join(Path::new(file_handle)
                .strip_prefix(prefix_path)?
                .with_extension("rs"));

            fs::create_dir_all(
                &out_path.parent().ok_or(NoserError::InvalidPath(out_path.to_owned()))?
            )?;

            File::create(out_path)?.write_all(result.as_bytes())?;
        }
        
        Ok(())
    }
}

#[derive(Debug)]
pub enum NoserError<I> {
    IOError(std::io::Error),
    ParserError(NomError<I>),
    PrefixError(std::path::StripPrefixError),
    EnvironmentVariableError(std::env::VarError),
    InvalidPath(std::path::PathBuf),
}

#[derive(Debug)]
pub enum NomError<I, E = u32> {
    Incomplete(nom::Needed),
    Error(NomContext<I, E>),
    Failure(NomContext<I, E>),
}

#[derive(Debug)]
pub enum NomContext<I, E = u32> {
    Code(I, nom::ErrorKind<E>)
}

impl<I1, I2> From<nom::Err<I1>> for NoserError<I2> 
    where I2: From<I1> 
{
    fn from(error: nom::Err<I1>) -> NoserError<I2> {
        NoserError::ParserError(match error {
            nom::Err::Incomplete(needed) => NomError::Incomplete(needed),
            nom::Err::Error(context) => NomError::Error(From::from(context)),
            nom::Err::Failure(context) => NomError::Failure(From::from(context)),
        })
    }
}

impl<I1, I2> From<nom::Context<I1>> for NomContext<I2>
    where I2: From<I1>
{
    fn from(from: nom::Context<I1>) -> NomContext<I2> {
        match from {
            nom::Context::Code(i, e) => NomContext::Code(From::from(i), e)
        }
    }
}

impl<I> From<std::path::StripPrefixError> for NoserError<I> {
    fn from(error: std::path::StripPrefixError) -> NoserError<I> {
        NoserError::PrefixError(error)
    }
}

impl<I> From<std::io::Error> for NoserError<I> {
    fn from(error: std::io::Error) -> NoserError<I> {
        NoserError::IOError(error)
    }
}

impl<I> From<std::env::VarError> for NoserError<I> {
    fn from(error: std::env::VarError) -> NoserError<I> {
        NoserError::EnvironmentVariableError(error)
    }
}

#[cfg(test)]
mod test {
    use *;

    #[test]
    fn test() {
        NoserCompiler::new()
            .base("test_out")
            .remove_prefix("src/schema")
            .file("src/schema/test.noser")
            .run()
            .expect("noserc failed to compile");
    }
}
