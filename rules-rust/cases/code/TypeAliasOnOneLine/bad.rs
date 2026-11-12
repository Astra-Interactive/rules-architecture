type KeepInventoryGate =
    FeatureGate<KeepInventoryModule, ConfigurationFeatureFlagReader<KeepInventoryConfiguration>>;

pub(crate) type Gate = FeatureGate<
    KeepInventoryModule,
    Reader,
>;

struct Feed;

impl Iterator for Feed {
    type Item =
        Pin<Box<dyn Future<Output = Notification> + Send>>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

trait Source {
    type Item: Clone =
        Vec<u8>;
}

some_macro! {
    type Gate =
        FeatureGate<Module, Reader>;
}

impl Feed for Gate {
    type Item<'a>
        = Pin<
            Box<dyn Future<Output = Result<Notification, FeedError>> + Send + 'a>,
        >
    where
        Self: 'a;
}

type Spread<
    'a,
    T,
>
    = Foo<'a, T>
where
    T: 'a;
