pub(crate) mod invoice {
    pub(crate) mod model {}
    pub(crate) mod bin {}
}

#[cfg(any(test, feature = "fake"))]
#[path = "../fixture"]
pub mod fixture {
    pub mod invoice {
        pub mod fake {
            pub use fake_invoice::FakeInvoice;
            mod fake_invoice;
        }
    }
}

#[cfg(test)]
#[path = "../test"]
mod test {}
