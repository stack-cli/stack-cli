use dioxus::prelude::*;

#[component]
pub fn Hero() -> Element {
    rsx! {
        section {
            h1 {
                class: "text-6xl max-w-3xl mx-auto text-center pb-6 font-semibold capitalize",
                "Supabase-style backend, simpler Kubernetes."
            }
            h2 {
                class: "subtitle max-w-3xl mx-auto text-center text-lg",
                "Stack turns a Kubernetes cluster into a backend platform: Postgres, auth, storage, REST, and realtime per app. You keep the simplicity of compose, get production parity, and avoid the cost curve of managed PaaS."
            }
        }
    }
}
