extern crate gitters;

use gitters::objects;
use gitters::revisions;

#[test]
fn full_sha1_resolves_to_self() {
    assert_eq!(
        Ok(objects::Name("4ddb0025ef5914b51fb835495f5259a6d962df21".to_string())),
        revisions::resolve("4ddb0025ef5914b51fb835495f5259a6d962df21"));
}

#[test]
fn partial_sha1_resolves_to_full_sha1_if_unambiguous() {
    assert_eq!(
        Ok(objects::Name("4ddb0025ef5914b51fb835495f5259a6d962df21".to_string())),
        revisions::resolve("4ddb0025e"));
}

#[test]
fn invalid_revision_does_not_resolve() {
    assert_eq!(
        Err(revisions::Error::InvalidRevision),
        revisions::resolve("invalid"));
}
