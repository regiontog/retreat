pub trait WithArena<'a, T> {
    fn with_arena(self, arena: &'a mut [u8]) -> ::Result<'a, T>;
}
