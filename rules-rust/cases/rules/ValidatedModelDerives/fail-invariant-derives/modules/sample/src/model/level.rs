#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub(crate) struct Level(u8);

impl TryFrom<u8> for Level {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 9 {
            return Err(());
        }
        Ok(Self(value))
    }
}
