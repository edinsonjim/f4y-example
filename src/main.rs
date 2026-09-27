mod app;
mod components;
mod db;
mod family;

use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    router::RouterBuilderDiscoverExt,
    runtime::RouterBuilderRuntimeExt,
};

#[tokio::main]
async fn main() {
    let db = db::connect().await;

    topcoat::start(router(db)).await.unwrap();
}

pub fn router(db: toasty::Db) -> topcoat::router::Router {
    topcoat::router::module_router!()
        .discover()
        .app_context(db)
        .assets(AssetBundle::load().unwrap())
        .runtime()
        .build()
}
