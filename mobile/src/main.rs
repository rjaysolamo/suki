use dioxus::prelude::*;

mod platform;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut result = use_signal(|| "Native connection has not been checked.".to_string());
    rsx! {
        document::Style { {include_str!("style.css")} }
        main {
            header { p { "BANKEROHAN PUBLIC MARKET" } h1 { "Suki Davao" } }
            section {
                h2 { "Your market, closer to home" }
                p { "Fresh finds from your local market, together in one delivery." }
                div { class: "notice", "Mobile foundation • Android verification" }
                p { "This build checks the Android runtime before account and marketplace features are introduced." }
                button {
                    onclick: move |_| result.set(platform::runtime_info().unwrap_or_else(|error| error)),
                    "Check Android connection"
                }
                p { role: "status", "{result}" }
            }
            footer { "Customer · Vendor · Courier · Admin" }
        }
    }
}
