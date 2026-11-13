use crate::workspace::model::{Dependency, Member};

impl Dependency {
    pub(crate) fn find_member<'members>(
        &self,
        members: &'members [Member],
    ) -> Option<&'members Member> {
        members.iter().find(|member| {
            let has_name = self.name == member.name;
            let has_path = self.path.as_deref() == Some(member.directory.as_str());
            has_name && has_path
        })
    }
}
