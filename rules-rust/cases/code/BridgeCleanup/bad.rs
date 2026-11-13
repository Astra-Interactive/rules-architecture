struct Guard;
impl Drop for Guard {
    fn drop(&mut self) { tokio::spawn(async {}); }
}

