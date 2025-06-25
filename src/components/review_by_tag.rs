use leptos::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::params::Params;
use leptos_router::hooks::use_params;
use crate::components::error_notification::ErrorNotification;
use crate::model;
use crate::components::review_cards::ReviewCards;
#[cfg(feature = "ssr")]
use crate::db::Database;

#[server(GetCardsByTag, "/api")]
async fn get_cards_by_tag(tag: String) -> Result<Vec<model::Flashcard>, ServerFnError> {
    let db = Database::get_instance().unwrap().lock().unwrap();
    db.all_cards(Some(tag)).map_err(ServerFnError::new)
}

#[derive(Params, PartialEq, Clone)]
struct ReviewByTagParams {
    tag: Option<String>,
}

#[component]
pub fn ReviewByTag() -> impl IntoView {
    let params = use_params::<ReviewByTagParams>();
    let tag = move || {
        params
            .read()
            .as_ref()
            .ok()
            .and_then(|params| params.tag.clone())
            .unwrap()
    };
    let (cards, set_cards) = signal(Vec::<model::Flashcard>::new());
    let (error, set_error) = signal(None::<String>);

    // Load cards
    Effect::new(move |_| {
        spawn_local(async move {
            match get_cards_by_tag(tag()).await {
                Ok(loaded_cards) => {
                    set_cards.set(loaded_cards);
                    set_error.set(None);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to load cards:\n {}", e)));
                }
            }
        });
    });

    view! { 
        <ReviewCards cards=cards />
        <ErrorNotification error=error />
    }
}
