use oca_parser_xls::xls_parser::oca::parse;
use oca_rust::state::validator::Validator;

#[test]
fn parse_oca_xls() {
    let result = parse(
        format!(
            "{}/tests/assets/oca_template.xlsx",
            env!("CARGO_MANIFEST_DIR")
        ),
        true,
        None,
        false,
        None,
    );

    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert_eq!(parsed.languages.len(), 1);

    let oca = parsed.oca_builder.finalize();
    assert_eq!(oca.capture_base.attributes.len(), 3);
    assert_eq!(oca.capture_base.flagged_attributes.len(), 1);
    assert_eq!(oca.overlays.len(), 8);

    let validator = Validator::new().enforce_translations(vec!["en".to_string()]);
    let validation_result = validator.validate(&oca);
    assert!(validation_result.is_ok());
}
