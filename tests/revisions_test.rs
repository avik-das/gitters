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
fn multiple_parent_specification_resolves_to_ancestor_sha1() {
    assert_eq!(
        Ok(objects::Name("3e6a5d72d0ce0af8402c7d467d1b754b61b79d16".to_string())),
        revisions::resolve("d7698dd^^^"));
}

#[test]
fn ancestor_specification_resolves_to_ancestor_sha1() {
    assert_eq!(
        Ok(objects::Name("3e6a5d72d0ce0af8402c7d467d1b754b61b79d16".to_string())),
        revisions::resolve("d7698dd~3"));
}

#[test]
fn branch_resolves_to_sha1() {
    assert_eq!(
        Ok(objects::Name("41cf28e8cac50f5cfeda40cfbfdd049763541c5a".to_string())),
        revisions::resolve("introduce-tests"));
}

#[test]
fn invalid_revision_does_not_resolve() {
    assert_eq!(
        Err(revisions::Error::InvalidRevision),
        revisions::resolve("invalid"));
}
