use leptos::*;
use leptos::prelude::*;
use gloo_timers::callback::Timeout;
use web_sys::console;

#[component]
pub fn ErrorNotification(
    #[prop(into)] error: Signal<Option<String>>,
) -> impl IntoView {
    let (show, set_show) = signal(false);
    let (error_message, set_error_message) = signal(String::new());

    // Watch for error changes
    Effect::new(move |_| {
        if let Some(err) = error.get() {
            console::error_1(&err.clone().into());
            set_error_message.set(err);
            set_show.set(true);
            
            // Auto-dismiss after 5 seconds
            Timeout::new(5000, move || set_show.set(false)).forget();
        }
    });

    view! {
        <Show when=move || show.get()>
            <div class="fixed bottom-4 right-4 bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded shadow-lg max-w-sm z-50">
                <div class="flex items-start">
                    <div class="flex-shrink-0">
                        <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                        </svg>
                    </div>
                    <div class="ml-3">
                        <p class="text-sm font-medium">{error_message}</p>
                    </div>
                    <div class="ml-auto pl-3">
                        <div class="-mx-1.5 -my-1.5">
                            <button
                                class="inline-flex bg-red-100 text-red-500 rounded-lg focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 p-1.5 hover:bg-red-200 transition-colors"
                                on:click=move |_| set_show.set(false)
                            >
                                <svg class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                    <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                                </svg>
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
} 