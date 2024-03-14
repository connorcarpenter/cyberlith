use cfg_if::cfg_if;

use ui::Ui;

cfg_if! {
    if #[cfg(feature = "read_bits")] {
        mod read;
    } else {}
}

cfg_if! {
    if #[cfg(feature = "write_bits")] {
        mod write;
    } else {}
}

pub fn write_bits(ui: &Ui) -> Vec<u8> {
    todo!()
}

pub fn read_bits(data: Vec<u8>) -> Ui {
    todo!()
}