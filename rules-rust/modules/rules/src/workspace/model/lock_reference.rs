pub(crate) struct LockReference<'lock> {
    pub(crate) name: &'lock str,
    pub(crate) version: Option<&'lock str>,
    pub(crate) source: Option<&'lock str>,
}
