use naia_serde::{FileBitWriter, SerdeInternal as Serde};

use ui::Ui;

use crate::bits::{UiAction, UiActionType};

pub fn write_bits(ui: &Ui) -> Vec<u8> {
    let actions = convert_ui_to_actions(ui);
    actions_to_bytes(actions)
}

fn convert_ui_to_actions(ui: &Ui) -> Vec<UiAction> {
    // write text color

    // write text icon AssetId

    // write styles

    // write nodes

    todo!()
}

fn actions_to_bytes(actions: Vec<UiAction>) -> Vec<u8> {
    let mut bit_writer = FileBitWriter::new();

    for action in actions {
        match action {
            UiAction::TextColor(r, g, b) => {
                UiActionType::TextColor.ser(&mut bit_writer);
                r.ser(&mut bit_writer);
                g.ser(&mut bit_writer);
                b.ser(&mut bit_writer);

            }
            UiAction::TextIconAssetId(asset_id) => {
                UiActionType::TextIconAssetId.ser(&mut bit_writer);
                asset_id.as_u32().ser(&mut bit_writer);
            }
            UiAction::Style() => {
                UiActionType::Style.ser(&mut bit_writer);
                todo!()
            }
            UiAction::Node() => {
                UiActionType::Node.ser(&mut bit_writer);
                todo!()
            }
        }
    }

    // continue bit
    UiActionType::None.ser(&mut bit_writer);

    bit_writer.to_bytes().to_vec()
}