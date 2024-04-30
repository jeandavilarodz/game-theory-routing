use crate::quadtree::Number;

#[derive(Clone, Debug)]
pub struct Point<U>
where
    U: Number,
{
    pub x: U,
    pub y: U,
}

impl<U> Point<U> where U: Number{
    pub fn new(x: U, y: U) -> Point<U>
    where
        U: Number,
    {
        Point { x, y }
    }
}

#[derive(Debug)]
pub struct Entry<U, V>
where
    U: Number,
{
    pub point: Point<U>,
    pub value: V,
}
impl<U, V> Entry<U, V> where U: Number {
    pub fn new(point: Point<U>, value: V) -> Self
    where
        U: Number,
    {
        Entry { point, value }
    }
}
