use naia_serde::SerdeInternal as Serde;

#[derive(Serde, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Debug)]
pub enum UserRole {
    Admin,
    Staff,
    Paid,
    Free,
}

