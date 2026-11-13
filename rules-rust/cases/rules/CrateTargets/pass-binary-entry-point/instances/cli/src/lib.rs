pub mod di {
    pub use root_module::RootModule;
    mod root_module;
}

#[cfg(test)]
#[path = "../test"]
mod test {
    mod di {
        mod root_module_test;
    }
}
