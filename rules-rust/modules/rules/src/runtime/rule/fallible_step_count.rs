#[derive(Clone, PartialEq, Eq)]
pub(crate) struct FallibleStepCount {
    sites: Vec<usize>,
    multiple: bool,
}

impl FallibleStepCount {
    pub(crate) fn empty() -> Self {
        Self {
            sites: Vec::new(),
            multiple: false,
        }
    }

    pub(crate) fn compute_maximum(&self) -> usize {
        if self.multiple {
            2
        } else {
            usize::from(!self.sites.is_empty())
        }
    }

    pub(crate) fn add(&mut self, site: usize) {
        if self.multiple {
            return;
        }
        self.multiple = self.sites.iter().any(|previous| *previous != site);
        self.sites.clear();
        if !self.multiple {
            self.sites.push(site);
        }
    }

    pub(crate) fn merge(mut self, other: Self) -> Self {
        self.multiple |= other.multiple;
        if self.multiple {
            self.sites.clear();
        } else {
            self.sites.extend(other.sites);
            self.sites.sort_unstable();
            self.sites.dedup();
        }
        self
    }
}
