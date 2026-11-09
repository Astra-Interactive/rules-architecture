mod di { struct ChildModule; impl ChildModule { fn new() -> Self { Self } } struct RootModule; impl RootModule { fn new() -> Self { let child = ChildModule::new(); Self } } }
