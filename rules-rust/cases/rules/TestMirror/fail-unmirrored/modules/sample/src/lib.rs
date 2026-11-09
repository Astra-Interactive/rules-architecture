pub(crate) mod invoice {
    pub(crate) mod model {
        mod invoice;
    }
}

#[cfg(test)]
#[path = "../test"]
mod test {
    mod invoice {
        mod model {
            mod missing_test;
        }
    }
}
