pub(crate) mod invoice {
    pub(crate) mod util {
        mod invoice_load;
    }
}

#[cfg(test)]
#[path = "../test"]
mod test {
    mod invoice {
        mod util {
            mod invoice_load_test;
        }
    }
}
