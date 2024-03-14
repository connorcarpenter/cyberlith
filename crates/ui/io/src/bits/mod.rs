use cfg_if::cfg_if;

use ui::Ui;

cfg_if! {
    if #[cfg(feature = "read_bits")] {
        mod read;

        pub fn read_bits(data: Vec<u8>) -> Ui {
            read::read_bits(data)
        }
    } else {}
}

cfg_if! {
    if #[cfg(feature = "write_bits")] {
        mod write;

        pub fn write_bits(ui: &Ui) -> Vec<u8> {
            write::write_bits(ui)
        }
    } else {}
}

use naia_serde::SerdeInternal as Serde;

use asset_id::AssetId;

// Actions
#[derive(Clone)]
pub(crate) enum UiAction {
    // r, g, b
    TextColor(u8, u8, u8),
    // assetid
    TextIconAssetId(AssetId),
    // style
    Style(),
    // node
    Node(),
}

#[derive(Serde, Clone, PartialEq)]
pub enum UiActionType {
    TextColor,
    TextIconAssetId,
    Style,
    Node,

    None,
}