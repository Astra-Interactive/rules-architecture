use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct FeatureGate<Owner, Reader> {
    owner: PhantomData<Owner>,
    reader: PhantomData<Reader>,
}
