use uuid::Uuid;

pub trait HasIdentity {
    #[must_use]
    fn id(&self) -> Uuid;
}
