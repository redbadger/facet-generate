#[derive(Facet)]
type AlsoString = String;

#[derive(Facet)]
#[facet(serialized_as = "String")]
struct Uuid(String);

#[derive(Facet)]
/// Unique identifier for an Account
type AccountUuid = Uuid;

#[derive(Facet)]
#[facet(serialized_as = "String")]
type ItemUuid = Uuid;
