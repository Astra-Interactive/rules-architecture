use crate::shape::model::{LineItem, Point};

pub(crate) trait Shape {
    fn position(&self) -> Point;
    fn line_items(&self) -> Vec<LineItem>;
}
