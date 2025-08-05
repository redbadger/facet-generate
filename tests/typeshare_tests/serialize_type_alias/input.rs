use facet::Facet;

type AlsoString = String;

#[derive(Facet)]
#[facet(serialized_as = "String")]
struct Uuid(String);

/// Unique identifier for an Account
type AccountUuid = Uuid;

type ItemUuid = Uuid;
