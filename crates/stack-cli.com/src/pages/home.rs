use crate::components::footer::Footer;
use crate::components::hero::Hero;
use crate::components::navigation::Section;
use crate::layouts::layout::Layout;
use dioxus::prelude::*;

pub fn home_page() -> String {
    let install_script =
        r#"curl -fsSL https://stack-cli.com/install.sh | bash"#;

    let features = vec![
        (
            "Just build the app",
            "Define your backend stack once and focus on product work instead of wiring services.",
        ),
        (
            "Local dev that matches prod",
            "Run the same stack on your laptop and in production, without separate compose or staging setups.",
        ),
        (
            "Architecture decided for you",
            "Postgres, auth, REST, realtime, storage, and ingress are ready per app namespace.",
        ),
        (
            "Deployments without the YAML maze",
            "One manifest declares everything; the operator reconciles it safely.",
        ),
        (
            "No surprise platform bills",
            "Run on your own VMs or clusters and scale up only when needed.",
        ),
    ];

    let benefits = vec![
        (
            "Supabase-style workflow on Kubernetes",
            "Get Postgres, auth, REST, realtime, and storage per app without managing a SaaS platform.",
        ),
        (
            "Simpler than hand-rolled K8s",
            "One manifest defines your app and backend services, so you skip bespoke Helm charts and wiring.",
        ),
        (
            "Lower cost than managed PaaS",
            "Run on your own VMs or clusters and scale only when you need it.",
        ),
        (
            "Dev equals prod",
            "The same CRDs and operator reconcile flow across local and production clusters.",
        ),
    ];

    let page = rsx! {
        Layout {
            title: "Stack",
            description: "Stack turns Kubernetes into a self-hosted PaaS so you can deploy apps without assembling operators by hand.",
            mobile_menu: None,
            section: Section::Home,

            div {
                class: "p-5 mt-16 mx-auto max-w-5xl",
                Hero {}

            }

            section {
                id: "install-stack",
                class: "mt-20 px-5",
                div {
                    class: "max-w-4xl mx-auto bg-base-200 border border-base-300 rounded-2xl p-8",
                    h2 {
                        class: "text-3xl font-semibold text-center",
                        "Install the Stack CLI"
                    }
                    p {
                        class: "mt-4 text-center text-base-content/80",
                        "Download the CLI and bring the Stack platform into any Kubernetes cluster with a single command."
                    }
                    div {
                        class: "mt-10",
                        pre {
                            class: "bg-black text-white text-sm rounded-xl p-5 overflow-x-auto",
                            code {
                                class: "language-bash",
                                "{install_script}"
                            }
                        }
                    }
                }
            }

            section {
                class: "mt-20 px-5",
                div {
                    class: "max-w-6xl mx-auto",
                    h2 {
                        class: "text-3xl font-semibold text-center",
                        "Everything a delivery team needs"
                    }
                    p {
                        class: "mt-4 text-center text-base-content/80",
                        "Stack installs a curated set of services so your applications are ready for production from day one."
                    }
                    div {
                        class: "mt-10 grid grid-cols-1 md:grid-cols-2 gap-6",
                        for (title, description) in features.iter() {
                            div {
                                class: "border border-base-300 rounded-2xl bg-base-200 p-6",
                                h3 {
                                    class: "text-xl font-semibold",
                                    "{title}"
                                }
                                p {
                                    class: "mt-2 text-base-content/80",
                                    "{description}"
                                }
                            }
                        }
                    }
                }
            }

            section {
                class: "mt-20 mb-24 px-5",
                div {
                    class: "max-w-5xl mx-auto",
                    h2 {
                        class: "text-3xl font-semibold text-center",
                        "Why teams choose Stack"
                    }
                    div {
                        class: "mt-10 grid grid-cols-1 md:grid-cols-2 gap-6",
                        for (title, description) in benefits.iter() {
                            div {
                                class: "border border-base-300 rounded-2xl bg-base-100 p-6",
                                h3 {
                                    class: "text-xl font-semibold",
                                    "{title}"
                                }
                                p {
                                    class: "mt-2 text-base-content/80",
                                    "{description}"
                                }
                            }
                        }
                    }
                }
            }

            Footer {}
        }
    };

    crate::render(page)
}
