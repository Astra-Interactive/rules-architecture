#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SaveUserResult {
    Success,
    AlreadyRegistered,
}
