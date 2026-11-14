#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Role {
    ProcMacro,
    EntryPoint,
    CoreUi,
    CorePlatform,
    CoreApi,
    Core,
    Contract,
    StorageContract,
    Api,
    Impl,
    ImplUi,
    Platform,
    Technology,
    Context,
}
