use leptos::prelude::*;
use thaw::{Toast, ToastBody, ToastIntent, ToastOptions, ToastTitle, ToasterInjection};

#[derive(Clone, Copy)]
pub struct ShowError {
    toaster: ToasterInjection,
}

impl ShowError {
    pub fn from_ctx() -> Self {
        let toaster = ToasterInjection::expect_context();
        Self { toaster }
    }

    pub fn show(&self, error: String) {
        self.toaster.dispatch_toast(
            move || {
                view! {
                    <Toast>
                        <ToastTitle>"Error"</ToastTitle>
                        <ToastBody>{error}</ToastBody>
                    </Toast>
                }
            },
            ToastOptions::default().with_intent(ToastIntent::Error),
        );
    }
}
