mod model {
    use engine::Context as FrameworkContext;
    use database::Row as DatabaseRow;
    use calendar::Moment as ClockMoment;
    pub struct Value {
        context: FrameworkContext,
        row: DatabaseRow,
        moment: ClockMoment,
    }
}
mod file {
    use engine::Context;
    use database::Row;
    use calendar::Moment;
    pub struct Gateway {
        context: Context,
        row: Row,
        moment: Moment,
    }
}
#[cfg(feature = "native")]
mod alternate {
    pub mod model {
        use bridge::ExportedContext as Alias;
        pub struct Wrapped { value: Alias }
    }
}
