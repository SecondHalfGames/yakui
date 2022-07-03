pub struct ZipLongest<A, B> {
    a: A,
    b: B,
}

impl<A, B, AI, BI> Iterator for ZipLongest<A, B>
where
    A: Iterator<Item = AI>,
    B: Iterator<Item = BI>,
{
    type Item = (Option<AI>, Option<BI>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let a = self.a.next();
        let b = self.b.next();

        if a.is_some() || b.is_some() {
            Some((a, b))
        } else {
            None
        }
    }
}

pub fn zip<A, B>(a: A, b: B) -> ZipLongest<A::IntoIter, B::IntoIter>
where
    A: IntoIterator,
    B: IntoIterator,
{
    ZipLongest {
        a: a.into_iter(),
        b: b.into_iter(),
    }
}
