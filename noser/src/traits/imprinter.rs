pub trait Imprinter<'a> {
    fn imprint(self, arena: &'a mut [u8]) -> ::Result<()>;
}
