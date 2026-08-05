#[cfg(unix)]
use std::ffi::OsString;

use super::*;

#[test]
fn runtime_value_takes_precedence_over_embedded_value() -> Result<()> {
    let value = configuration_value("SETTING", Ok("runtime".to_owned()), Some("embedded"))?;

    assert_eq!(value, "runtime");
    Ok(())
}

#[test]
fn embedded_value_is_used_when_runtime_value_is_missing() -> Result<()> {
    let value = configuration_value("SETTING", Err(VarError::NotPresent), Some("embedded"))?;

    assert_eq!(value, "embedded");
    Ok(())
}

#[test]
fn empty_runtime_value_is_rejected_instead_of_using_embedded_value() {
    let error = configuration_value("SETTING", Ok(String::new()), Some("embedded"))
        .expect_err("an empty runtime override must be rejected");

    assert_eq!(error.to_string(), "SETTING must not be empty");
}

#[test]
fn empty_embedded_value_is_rejected() {
    let error = configuration_value("SETTING", Err(VarError::NotPresent), Some(""))
        .expect_err("an empty embedded value must be rejected");

    assert_eq!(
        error.to_string(),
        "SETTING must be set in the environment, .env file, or embedded at build time"
    );
}

#[test]
fn missing_runtime_and_embedded_values_are_rejected() {
    let error = configuration_value("SETTING", Err(VarError::NotPresent), None)
        .expect_err("a missing setting must be rejected");

    assert_eq!(
        error.to_string(),
        "SETTING must be set in the environment, .env file, or embedded at build time"
    );
}

#[cfg(unix)]
#[test]
fn non_unicode_runtime_value_is_rejected_instead_of_using_embedded_value() {
    use std::os::unix::ffi::OsStringExt;

    let error = configuration_value(
        "SETTING",
        Err(VarError::NotUnicode(OsString::from_vec(vec![0xff]))),
        Some("embedded"),
    )
    .expect_err("a non-Unicode runtime override must be rejected");

    assert_eq!(error.to_string(), "SETTING must contain valid Unicode");
}
