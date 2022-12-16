pub struct LoadingLinkedList<'a, D, T> {
    load_next: fn(&D) -> T,
    data: &'a D
}

impl<'a, D, T> LoadingLinkedList<'a, D, T> {
    pub fn new(data: &'a D, load_next: fn(&D) -> T) -> Self {
        return LoadingLinkedList {
            load_next,
            data
        }
    }

    pub fn next(&mut self) -> T {
        return (self.load_next)(&mut self.data);
    }
}