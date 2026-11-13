pub(crate) mod shape {
    pub(crate) mod api {
        pub(crate) use shape::Shape;
        mod shape;
    }
    pub(crate) mod model {
        pub(crate) use line_item::LineItem;
        pub(crate) use point::Point;
        pub(crate) use product_id::ProductId;
        mod line_item;
        mod point;
        mod product_id;
    }
    pub(crate) mod util {
        pub(crate) use label_split::rotate_label;
        mod label_split;
    }
}
