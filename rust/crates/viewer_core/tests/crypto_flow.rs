use viewer_core::wire::ViewerErrorCode;
use viewer_core::ViewerError;

#[test]
fn invalid_password_maps_to_expected_error_code() {
    let error = ViewerError::InvalidPassword;
    let response = error.to_error_response();

    assert_eq!(response.code, ViewerErrorCode::InvalidPassword);
    assert!(response.message.contains("invalid password"));
}

#[test]
fn unsupported_encryption_maps_to_expected_error_code() {
    let error = ViewerError::UnsupportedEncryption;
    let response = error.to_error_response();

    assert_eq!(response.code, ViewerErrorCode::UnsupportedEncryption);
    assert!(response.message.contains("unsupported encryption"));
}
