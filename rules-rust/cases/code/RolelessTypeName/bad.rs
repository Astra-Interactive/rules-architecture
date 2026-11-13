struct UserValidator;
struct UserFactory;
struct CacheWarmUp;
struct SchemaMigration;
trait MissingFactory: Send + Sync { fn invoke(&self); }
