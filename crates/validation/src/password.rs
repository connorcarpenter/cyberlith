use crate::{ValidationType, Validator};

pub struct PasswordValidation;

impl Validator for PasswordValidation {

    fn min_length() -> usize {
        8
    }

    fn max_length() -> usize {
        128
    }

    fn validation_type() -> ValidationType {
        ValidationType::Password
    }
}