use dioxus::prelude::*;

#[component]
pub fn Hero() -> Element {
    rsx! {
        section {
            h1 {
                class: "text-6xl max-w-3xl mx-auto text-center pb-6 font-semibold capitalize",
                "Production-ready infrastructure for self-hosted web applications"
            }
            h2 {
                class: "subtitle max-w-3xl mx-auto text-center text-lg",
                "Stack is an open-source, Kubernetes-native alternative to Heroku and Supabase. It gives you a database, auth, APIs, storage, and deployment as a single, self-hosted stack."
            }
        }
    }
}
