use naia_serde::{BitReader, SerdeInternal as Serde, SerdeErr};

use asset_id::AssetId;
use ui::Ui;

use crate::bits::{UiAction, UiActionType};

pub fn read_bits(data: Vec<u8>) -> Ui {
    let actions = bytes_to_actions(data).unwrap();
    convert_actions_to_ui(actions)
}

fn convert_actions_to_ui(actions: Vec<UiAction>) -> Ui {
    // read text color

    // read text icon AssetId

    // read styles

    // read nodes

    todo!()
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
                actions.push(UiAction::Style());
            }
            UiActionType::Node => {
                actions.push(UiAction::Node());
            }
            UiActionType::None => {
                break;
            }
        }
    }

    Ok(actions)
}