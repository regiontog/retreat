pub trait Imprinter<'a> {
    type OnSuccess;

    fn imprint(&self, arena: &'a mut [u8]) -> ::Result<Self::OnSuccess>;

    fn imprint_disregard_result(&self, arena: &'a mut [u8]) -> ::Result<()>
    where
        Self: Sized,
    {
        self.imprint(arena).map(|_| ())
    }
}
