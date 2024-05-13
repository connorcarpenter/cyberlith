use crate::{ValidationType, Validator};

pub struct EmailValidation;

impl Validator for EmailValidation {

    fn min_length() -> usize {
        3
    }

    fn max_length() -> usize {
        254
    }

    fn validation_type() -> ValidationType {
        ValidationType::Email
    }
}