use naia_serde::{BitReader, SerdeInternal as Serde, SerdeErr};

use asset_id::AssetId;
use asset_render::{AssetHandle, IconData};
use render_api::base::Color;
use ui::Ui;

use crate::bits::{UiAction, UiActionType, UiNodeBits, UiStyleBits};

pub fn read_bits(data: Vec<u8>) -> Ui {
    let actions = bytes_to_actions(data).unwrap();
    convert_actions_to_ui(actions)
}

fn convert_actions_to_ui(actions: Vec<UiAction>) -> Ui {
    let mut ui = Ui::new();

    for action in actions {
        match action {
            UiAction::TextColor(r, g, b) => {
                let color = Color::new(r, g, b);
                ui.set_text_color(color);
            }
            UiAction::TextIconAssetId(asset_id) => {
                let asset_handle = AssetHandle::<IconData>::new(asset_id);
                ui.set_text_icon_handle(&asset_handle);
            }
            UiAction::Style(style) => {
                todo!()
            }
            UiAction::Node(node) => {
                todo!()
            }
        }
    }

    ui
}

fn bytes_to_actions(bytes: Vec<u8>) -> Result<Vec<UiAction>, SerdeErr> {
    let mut bit_reader = BitReader::new(&bytes);
    let bit_reader = &mut bit_reader;
    let mut actions = Vec::new();

    loop {
        let action_type = UiActionType::de(bit_reader)?;

        match action_type {
            UiActionType::TextColor => {
                let r = u8::de(bit_reader)?;
                let g = u8::de(bit_reader)?;
                let b = u8::de(bit_reader)?;
                actions.push(UiAction::TextColor(r, g, b));
            }
            UiActionType::TextIconAssetId => {
                let val = u32::de(bit_reader)?;
                let asset_id = AssetId::from_u32(val).unwrap();
                actions.push(UiAction::TextIconAssetId(asset_id));
            }
            UiActionType::Style => {
                let style = UiStyleBits::de(bit_reader)?;
                actions.push(UiAction::Style(style));
            }
            UiActionType::Node => {
                let node = UiNodeBits::de(bit_reader)?;
                actions.push(UiAction::Node(node));
            }
            UiActionType::None => {
                break;
            }
        }
    }

    Ok(actions)
}