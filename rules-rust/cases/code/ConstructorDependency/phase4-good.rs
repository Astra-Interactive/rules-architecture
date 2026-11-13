trait Repository {} struct Concrete; impl Repository for Concrete {} impl Concrete { fn new() -> Self { Concrete } }
mod di { struct OwnerModule; impl OwnerModule { fn new() -> Self { let dependency = super::Concrete::new(); Self } } }
