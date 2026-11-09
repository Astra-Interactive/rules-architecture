#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Role {
    Tooling,
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
