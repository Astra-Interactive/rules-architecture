pub(crate) mod sample {}

#[cfg(any(test, feature = "fake"))]
#[path = "../fixture"]
pub mod fixture {
    pub mod sample {
        pub mod fake {
            mod fake_sample;
        }
    }
}

#[cfg(test)]
#[path = "../test"]
mod test {
    mod sample {
        mod usecase {
            mod sample_test;
        }
    }
}
