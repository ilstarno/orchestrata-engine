pub(super) struct Budget {
    pub used: u32,
    pub limit: u32,
}
impl Budget {
    pub fn step(&mut self) -> Result<(), ()> {
        if self.used == self.limit {
            return Err(());
        }
        self.used += 1;
        Ok(())
    }
}
