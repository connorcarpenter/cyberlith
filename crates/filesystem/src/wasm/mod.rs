mod functions;
mod types;

pub use functions::*;
pub use types::*;

// NOTE:
// will use Navigator, StorageManager from web_sys

// from load_asset in Bevy 0.11:
// let id = futures_lite::future::block_on(asset_server.load_async(path.clone(), true)).unwrap();

// from asset_server in Bevy 0.11;
// async fn load_async(
//         &self,
//         asset_path: AssetPath<'_>,
//         force: bool,
//     ) -> Result<AssetPathId, AssetServerError> {
//         let asset_path_id: AssetPathId = asset_path.get_id();
//
//         let set_asset_failed = || {
//             let mut asset_sources = self.server.asset_sources.write();
//             let source_info = asset_sources
//                 .get_mut(&asset_path_id.source_path_id())
//                 .expect("`AssetSource` should exist at this point.");
//             source_info.load_state = LoadState::Failed;
//         };
//
//         // load the asset bytes
//         let bytes = match self.asset_io().load_path(asset_path.path()).await {
//             Ok(bytes) => bytes,
//             Err(err) => {
//                 set_asset_failed();
//                 return Err(AssetServerError::AssetIoError(err));
//             }
//         };
//     }

// from wasm_asset_io in Bevy 0.11:
// impl AssetIo for WasmAssetIo {
//     fn load_path<'a>(&'a self, path: &'a Path) -> BoxedFuture<'a, Result<Vec<u8>, AssetIoError>> {
//         Box::pin(async move {
//             let path = self.root_path.join(path);
//             let window = web_sys::window().unwrap();
//             let resp_value = JsFuture::from(window.fetch_with_str(path.to_str().unwrap()))
//                 .await
//                 .unwrap();
//             let resp: Response = resp_value.dyn_into().unwrap();
//             let data = JsFuture::from(resp.array_buffer().unwrap()).await.unwrap();
//             let bytes = Uint8Array::new(&data).to_vec();
//             Ok(bytes)
//         })
//     }
//}
// BoxedFuture in Bevy 0.11
//
// #[cfg(not(target_arch = "wasm32"))]
// pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
//
// #[allow(missing_docs)]
// #[cfg(target_arch = "wasm32")]
// pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
