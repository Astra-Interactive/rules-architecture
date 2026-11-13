struct User;
trait UserFactory: Send + Sync { fn invoke(&self) -> User; }
mod di { struct Module; impl super::UserFactory for Module { fn invoke(&self) -> super::User { super::User } } }
mod warmup { struct CacheWarmUp; }
mod migration { struct SchemaMigration; }
