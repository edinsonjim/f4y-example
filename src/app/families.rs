use crate::components::{
    button::{ButtonVariant, button},
    card::{card, card_content, card_description, card_header, card_title},
    table::{table, table_body, table_cell, table_head, table_header, table_row},
};
use topcoat::{
    Result,
    router::page,
    view::{View, view},
};

#[page("/families")]
async fn index() -> Result<impl View> {
    Ok(view! {
        <main class="mx-auto flex max-w-5xl flex-col gap-6 p-8">
            card(
                card_header(
                    card_title("Families")
                    card_description("CRUD built with Topcoat 0.9, Toasty and SQLite.")
                )
                card_content(
                    table(
                        table_header(
                            table_row(
                                table_head("ID")
                                table_head("Name")
                                table_head("Summary")
                                table_head("Actions")
                            )
                        )
                        table_body(
                            table_row(
                                table_cell("1")
                                table_cell("The Simpsons")
                                table_cell("The Springfield family")
                                table_cell(button(variant: ButtonVariant::Outline, "Edit"))
                            )
                        )
                    )
                )
            )
        </main>
    })
}
