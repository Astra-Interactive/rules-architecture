use crate::catalogue::model::RetiredRule;

impl RetiredRule {
    pub(crate) fn describe_retirement(&self) -> String {
        match self.replacement {
            Some(replacement) => format!("use `{replacement}`"),
            None => "it has no successor".to_owned(),
        }
    }
}
