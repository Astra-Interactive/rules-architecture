#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CatalogueEntry {
    pub(crate) id: &'static str,
    pub(crate) phase: u8,
    pub(crate) anchors: AnchorKinds,
    pub(crate) reads_platform_crates: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AnchorKinds {
    pub(crate) node: bool,
    pub(crate) module: bool,
    pub(crate) file: bool,
    pub(crate) krate: bool,
    pub(crate) workspace: bool,
}
