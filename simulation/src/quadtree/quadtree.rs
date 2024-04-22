use yew::{Html, html};
use super::{box2d::Box2d};
use super::types::{Entry, Point};

pub struct QuadTree<V>
{
    capacity: usize,
    boundary: Box2d,
    entries: Vec<Entry<f64, V>>,

    north_west: Option<Box<QuadTree<V>>>,
    north_east: Option<Box<QuadTree<V>>>,
    south_west: Option<Box<QuadTree<V>>>,
    south_east: Option<Box<QuadTree<V>>>,
}

impl<V> QuadTree<V>
where
    V: Clone,
{
    pub fn new(boundary: Box2d, capacity: usize) -> Self {
        QuadTree {
            capacity,
            boundary,
            entries: Vec::new(),
            north_west: None,
            north_east: None,
            south_west: None,
            south_east: None,
        }
    }

    pub fn insert(&mut self, point: Point<f64>, value: V) -> bool {
        // if point is not in boundary, return false
        if !self.boundary.contains(&point) {
            return false;
        }

        // if there is space in the current quadtree, add the point and return true
        if self.entries.len() < self.capacity && self.north_east.is_none() {
            self.entries.push(Entry::new(point, value));
            return true;
        }

        // if there is no space in the current quadtree, subdivide and try to insert the point
        if self.north_east.is_none() {
            self.subdivide();
        }

        // try to insert the point in the sub quadtree
        if self.north_west.as_mut().unwrap().insert(point.clone(), value.clone()) {
            return true;
        }
        if self.north_east.as_mut().unwrap().insert(point.clone(), value.clone()) {
            return true;
        }
        if self.south_west.as_mut().unwrap().insert(point.clone(), value.clone()) {
            return true;
        }
        if self.south_east.as_mut().unwrap().insert(point.clone(), value) {
            return true;
        }

        // Nothing can be done, return false
        false
    }

    fn subdivide(&mut self) {
        let sub_boxes = self.boundary.subdivide();

        self.north_west = Some(Box::new(QuadTree::new(sub_boxes[0].clone(), self.capacity)));
        self.north_east = Some(Box::new(QuadTree::new(sub_boxes[1].clone(), self.capacity)));
        self.south_east = Some(Box::new(QuadTree::new(sub_boxes[2].clone(), self.capacity)));
        self.south_west = Some(Box::new(QuadTree::new(sub_boxes[3].clone(), self.capacity)));
    }

    pub fn query_range(&self, range: Box2d) -> Vec<Entry<f64, &V>> {
        let mut entries = Vec::new();

        if !self.boundary.intersects_box(&range) {
            return entries;
        }

        for entry in &self.entries {
            if range.contains(&entry.point) {
                entries.push(Entry::new(entry.point.clone(), &entry.value));
            }
        }

        if self.north_east.is_none() {
            return entries;
        }

        // query the sub quadtree
        entries.append(&mut self.north_west.as_ref().unwrap().query_range(range.clone()));
        entries.append(&mut self.north_east.as_ref().unwrap().query_range(range.clone()));
        entries.append(&mut self.south_west.as_ref().unwrap().query_range(range.clone()));
        entries.append(&mut self.south_east.as_ref().unwrap().query_range(range));

        entries
    }

    pub fn render(&self) -> Html {
        html! {
            <svg>
            { self.boundary.render() }
            if self.north_west.is_some() {
                <g>
                { self.north_west.as_ref().unwrap().render() }
                { self.north_east.as_ref().unwrap().render() }
                { self.south_west.as_ref().unwrap().render() }
                { self.south_east.as_ref().unwrap().render() }
                </g>
            }
            </svg>
        }
    }
}
