use std::fmt::Debug;

use naia_serde::{SerdeInternal as Serde, UnsignedInteger, UnsignedVariableInteger};

#[derive(Serde, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Timestamp {
    day: UnsignedInteger<5>,
    month: UnsignedInteger<4>,
    year: UnsignedVariableInteger<2>, // years past 2024
    hour: UnsignedInteger<4>,
    minute: UnsignedInteger<6>,
    pm: bool,
}

impl Timestamp {
    pub fn new(day: u8, month: u8, year: u16, hour: u8, minute: u8, pm: bool) -> Self {
        Self {
            day: day.into(),
            month: month.into(),
            year: year.into(),
            hour: hour.into(),
            minute: minute.into(),
            pm,
        }
    }

    pub fn to_string(&self) -> String {

        let am_pm_string = if self.pm { "PM" } else { "AM" };

        format!(
            "{:02}/{:02}/{:02} {:02}:{:02} {}",
            self.month.to::<u8>(),
            self.day.to::<u8>(),
            self.year.to::<u16>() + 2024,
            self.hour.to::<u8>(),
            self.minute.to::<u8>(),
            am_pm_string,
        )
    }
}

impl Debug for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        impl Timestamp {
            pub fn now() -> Self {
                use chrono::{Local, Timelike, Datelike};

                // Get the current date and time
                let now = Local::now();

                // Extract date and time components
                let day = now.day();
                let month = now.month();
                let year = now.year() - 2024;
                let hour_24 = now.hour();
                let minute = now.minute();

                // Determine AM/PM
                let pm = hour_24 >= 12;
                let hour = if hour_24 % 12 == 0 { 12 } else { hour_24 % 12 };

                Self::new(day as u8, month as u8, year as u16, hour as u8, minute as u8, pm)
            }
        }
    } else {}
}