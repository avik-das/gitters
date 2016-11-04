extern crate gitters;

use gitters::objects;

#[test]
fn reads_header_for_valid_object() {
    let name = objects::Name("4ddb0025ef5914b51fb835495f5259a6d962df21".to_string());
    match objects::read_header(&name) {
        Ok(header) => {
            assert_eq!(
                objects::Header { object_type: objects::Type::Commit, content_length: 384 },
                header)
        },
        Err(err) => panic!("unexpected error: {}", err),
    }
}
