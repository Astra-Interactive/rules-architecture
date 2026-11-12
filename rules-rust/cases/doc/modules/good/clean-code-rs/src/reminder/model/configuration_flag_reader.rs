use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ConfigurationFlagReader<Configuration>(PhantomData<Configuration>);
