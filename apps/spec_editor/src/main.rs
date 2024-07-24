mod avatar;
mod catalog;
mod animated_model;

use asset_id::{AssetId, ETag};
use spec::AnimatedModel;

use crate::animated_model::write_to_file;

fn main() {
    load_spec(avatar::define()); // avatar animated model
}

pub(crate) fn load_spec(
    spec_define: (String, AssetId, ETag, AnimatedModel),
) {
    let (spec_name, spec_asset_id, spec_etag, spec) = spec_define;

    write_to_file(&spec_name, &spec_asset_id, &spec_etag, spec);
}