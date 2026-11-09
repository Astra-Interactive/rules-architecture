mod account { mod api { pub(crate) trait Reader {} } }
#[cfg(test)] mod test { mod account { mod fake { pub(crate) struct FakeReader; impl crate::account::api::Reader for FakeReader {} } } }
