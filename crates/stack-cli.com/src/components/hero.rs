use dioxus::prelude::*;

#[component]
pub fn Hero() -> Element {
    rsx! {
        section {
            h1 {
                class: "text-6xl max-w-3xl mx-auto text-center pb-6 font-semibold capitalize",
                "Supabase-like developer experience. Kubernetes-native under the hood."
            }
            h2 {
                class: "subtitle max-w-3xl mx-auto text-center text-lg",
                "Stack CLI gives you an opinionated backend stack—Postgres, auth, storage, and services—deployed as a single application unit on Kubernetes. It’s designed for local development and VM-based clusters, without managed cloud lock-in or platform sprawl."
            }
        }
    }
}
