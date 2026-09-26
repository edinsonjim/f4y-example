use topcoat::{
    Result,
    font::fontsource::fontsource_font,
    router::{Slot, layout, page},
    tailwind,
    view::{View, view},
};

mod families;

/// The root document. Every page in this module is rendered inside it.
#[layout("/")]
async fn document(slot: Slot<'_>) -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>"Families"</title>
                topcoat::font::link(font: fontsource_font!(GEIST))
                <link rel="stylesheet" href=(tailwind::stylesheet!())>
                topcoat::dev::script()
                topcoat::runtime::script()
            </head>
            <body>
                (slot)
            </body>
        </html>
    })
}

#[page("/")]
async fn home() -> Result<impl View> {
    Ok(view! {
        <main class="mx-auto max-w-5xl p-8">
            <h1 class="text-2xl font-bold">"Families"</h1>
        </main>
    })
}
