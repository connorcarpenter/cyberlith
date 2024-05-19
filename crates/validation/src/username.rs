use crate::{ValidationType, Validator};

pub struct UsernameValidation;

impl Validator for UsernameValidation {
    fn min_length() -> usize {
        5
    }

    fn max_length() -> usize {
        15
    }

    fn validation_type() -> ValidationType {
        ValidationType::Username
    }
}
