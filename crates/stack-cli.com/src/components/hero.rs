use dioxus::prelude::*;

#[component]
pub fn Hero() -> Element {
    rsx! {
        section {
            h1 {
                class: "text-6xl max-w-3xl mx-auto text-center pb-6 font-semibold capitalize",
                "Self-hosting that doesn’t feel like self-hosting."
            }
            h2 {
                class: "subtitle max-w-3xl mx-auto text-center text-lg",
                "Stack is an open-source, Kubernetes-native alternative to Heroku, and Supabase—deploy apps, databases, auth, storage, and realtime as a single stack on your own infrastructure."
            }
        }
    }
}
