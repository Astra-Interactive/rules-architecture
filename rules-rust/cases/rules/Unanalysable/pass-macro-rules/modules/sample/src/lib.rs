macro_rules! sample_macro {
    () => {};
}
pub(crate) mod sample {
    pub(crate) mod util {
        mod nested;
    }
}

#[cfg(test)]
#[path = "../test"]
mod test {}
