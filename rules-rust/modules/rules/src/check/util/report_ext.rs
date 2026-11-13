use crate::check::model::Location;
use crate::check::model::Report;
use std::process::ExitCode;

impl Report {
    fn format_location(location: &Location) -> String {
        match location.line {
            Some(line) => format!("{}:{line}", location.path),
            None => location.path.clone(),
        }
    }

    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }

    pub fn to_violation_lines(&self) -> String {
        self.violations
            .iter()
            .map(|finding| {
                format!(
                    "{}\t{}\t{}\t{}\n",
                    finding.rule_id.0,
                    Self::format_location(&finding.location),
                    finding.subject,
                    finding.message
                )
            })
            .collect()
    }

    pub fn to_policy_lines(&self) -> String {
        self.accepted
            .iter()
            .map(|accepted| {
                format!(
                    "{}\t{}\t{}\t{}\n",
                    accepted.rule_id.0,
                    Self::format_location(&accepted.location),
                    accepted.subject,
                    accepted.reason
                )
            })
            .collect()
    }

    pub fn to_exit_code(&self) -> ExitCode {
        if self.is_clean() {
            return ExitCode::SUCCESS;
        }
        ExitCode::from(1)
    }
}
