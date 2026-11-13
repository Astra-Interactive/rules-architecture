pub mod ledger {
    pub mod api {
        pub use ledger_reader::LedgerReader;
        mod ledger_reader;
    }
    pub mod usecase {
        pub use count_ledger_use_case::CountLedgerUseCase;
        mod count_ledger_use_case;
    }
}

#[cfg(any(test, feature = "fake"))]
#[path = "../fixture"]
pub mod fixture {
    pub mod ledger {
        pub mod fake {
            pub use fake_ledger_reader::FakeLedgerReader;
            mod fake_ledger_reader;
        }
    }
}

#[cfg(test)]
#[path = "../test"]
mod test {
    mod ledger {
        mod usecase {
            mod count_ledger_use_case_test;
        }
    }
}
