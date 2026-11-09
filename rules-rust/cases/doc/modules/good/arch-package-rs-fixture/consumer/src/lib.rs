pub(crate) mod invoice {
    pub(crate) mod command {
        pub(crate) use invoice_count_command::InvoiceCountCommand;
        mod invoice_count_command;
    }
}

#[cfg(test)]
#[path = "../test"]
mod test {
    mod invoice {
        mod command {
            mod invoice_count_command_test;
        }
    }
}
