trait Repository {} struct Concrete; impl Repository for Concrete {} impl Concrete { fn new() -> Self { Self } }
struct Consumer; impl Consumer { fn new() -> Self { let dependency = Concrete::new(); Self } }
