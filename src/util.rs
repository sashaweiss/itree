pub struct PutBack<I: Iterator> {
    it: I,
    slot: Option<I::Item>,
}

impl<I: Iterator> Iterator for PutBack<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(s) = self.slot.take() {
            Some(s)
        } else {
            self.it.next()
        }
    }
}

impl<I: Iterator> PutBack<I> {
    pub fn new(it: I) -> Self {
        Self { it, slot: None }
    }

    /// Put an item back into the iterator. Panics if an item is already on-deck.
    pub fn put_back(&mut self, s: I::Item) {
        if self.slot.is_none() {
            self.slot = Some(s);
        } else {
            panic!("PutBack already had a put-back item on-deck!");
        }
    }
}
