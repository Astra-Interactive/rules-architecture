use crate::workspace::api::RustfmtReader;
use crate::workspace::model::RustfmtFacts;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

pub(crate) struct CargoRustfmtReader {
    rustfmt: PathBuf,
}

impl CargoRustfmtReader {
    pub(crate) fn new(rustfmt: PathBuf) -> Self {
        Self { rustfmt }
    }

    fn run_rustfmt(&self, argument: &str) -> Option<String> {
        let output = Command::new(&self.rustfmt).arg(argument).output().ok()?;
        if !output.status.success() {
            return None;
        }
        String::from_utf8(output.stdout).ok()
    }

    fn parse_default_keys(text: &str) -> BTreeSet<String> {
        text.lines()
            .filter_map(|line| line.split_once('='))
            .map(|(key, _)| key.trim().to_owned())
            .filter(|key| !key.is_empty())
            .collect()
    }

    fn parse_stable_keys(text: &str) -> BTreeSet<String> {
        text.lines()
            .filter_map(|line| {
                let mut words = line.split_whitespace();
                let key = words.next()?;
                let shape = words.next()?;
                let is_option = shape.starts_with('<') || shape.starts_with('[');
                is_option.then(|| key.to_owned())
            })
            .collect()
    }
}

impl RustfmtReader for CargoRustfmtReader {
    fn read_rustfmt_facts(&self) -> RustfmtFacts {
        let binary = self.rustfmt.to_string_lossy().into_owned();
        let Some(defaults) = self.run_rustfmt("--print-config=default") else {
            return RustfmtFacts::Missing { binary };
        };
        let Some(help) = self.run_rustfmt("--help=config") else {
            return RustfmtFacts::Missing { binary };
        };
        let stable_keys = Self::parse_stable_keys(&help);
        let nightly_only_keys = Self::parse_default_keys(&defaults)
            .into_iter()
            .filter(|key| !stable_keys.contains(key))
            .collect();
        RustfmtFacts::Queried { nightly_only_keys }
    }
}
