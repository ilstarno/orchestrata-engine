use super::{
    ModelError, boolean_field::BooleanField, field_type::FieldType, id_field::IdField,
    input_binding::InputBinding, unsigned_field::UnsignedField, validated_record::ValidatedRecord,
};
use std::collections::BTreeMap;
#[test]
fn field_roles_and_record_ownership_are_checked() {
    let schema = || InputBinding {
        field: "rows".into(),
        fields: BTreeMap::from([
            ("key".into(), FieldType::Id),
            ("number".into(), FieldType::U32),
            ("flag".into(), FieldType::Bool),
        ]),
    };
    let inputs = BTreeMap::from([("left".into(), schema()), ("right".into(), schema())]);
    let id = IdField::bind(&inputs, "left", "key".into()).unwrap();
    let number = UnsignedField::bind(&inputs, "left", "number".into()).unwrap();
    let flag = BooleanField::bind(&inputs, "left", "flag".into()).unwrap();
    assert_eq!(
        UnsignedField::bind(&inputs, "left", "flag".into()).unwrap_err(),
        ModelError::UnsupportedSemantics
    );
    assert_eq!(
        IdField::bind(&inputs, "left", "absent".into()).unwrap_err(),
        ModelError::UnsupportedSemantics
    );
    let data = r#"{"key":"record-a","number":7,"flag":false}"#;
    let row = ValidatedRecord::parse("left", &inputs["left"], serde_json::from_str(data).unwrap())
        .unwrap();
    assert_eq!(id.read(&row).unwrap().clone().into_string(), "record-a");
    assert_eq!(number.read(&row), Ok(7));
    assert_eq!(flag.read(&row), Ok(false));
    let other = ValidatedRecord::parse(
        "right",
        &inputs["right"],
        serde_json::from_str(data).unwrap(),
    )
    .unwrap();
    assert_eq!(number.read(&other), Err(ModelError::InvalidInput));
}
#[test]
fn scalar_validation_preserves_strict_input_types() {
    let binding = InputBinding {
        field: "rows".into(),
        fields: BTreeMap::from([("quantity".into(), FieldType::U32)]),
    };
    for bad in [
        r#"{"quantity":"7"}"#,
        r#"{"quantity":true}"#,
        r#"{"quantity":-1}"#,
        r#"{"quantity":1.2}"#,
        r#"{"quantity":1000001}"#,
        r#"{"quantity":null}"#,
        r#"{"quantity":[]}"#,
        r#"{"other":7}"#,
    ] {
        assert!(matches!(
            ValidatedRecord::parse("x", &binding, serde_json::from_str(bad).unwrap()),
            Err(ModelError::InvalidInput)
        ));
    }
}
