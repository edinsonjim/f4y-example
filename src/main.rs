mod app;
mod components;

use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    router::RouterBuilderDiscoverExt,
    runtime::RouterBuilderRuntimeExt,
};

#[tokio::main]
async fn main() {
    topcoat::start(router()).await.unwrap();
}

pub fn router() -> topcoat::router::Router {
    topcoat::router::module_router!()
        .discover()
        .assets(AssetBundle::load().unwrap())
        .runtime()
        .build()
}
