extern crate gitters;

use gitters::revisions;

#[test]
fn full_sha1_resolves_to_self() {
    assert_eq!(
        Ok("4ddb0025ef5914b51fb835495f5259a6d962df21"),
        revisions::resolve("4ddb0025ef5914b51fb835495f5259a6d962df21"));
}

#[test]
fn invalid_revision_does_not_resolve() {
    assert_eq!(
        Err(revisions::RevisionError::InvalidRevision),
        revisions::resolve("invalid"));
}
