pub struct ModelState {}
pub struct StateIterator {}

impl StateIterator {
    pub fn new(
        graph: crate::ForwardGraph,
        from: Option<demes::Time>,
        until: Option<demes::Time>,
    ) -> Self {
        todo!()
    }
}

impl Iterator for StateIterator {
    type Item = ModelState;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
