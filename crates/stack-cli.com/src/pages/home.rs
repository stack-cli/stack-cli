use crate::components::footer::Footer;
use crate::components::hero::Hero;
use crate::components::navigation::Section;
use crate::layouts::layout::Layout;
use dioxus::prelude::*;

pub fn home_page() -> String {
    let install_script = r#"curl -fsSL https://stack-cli.com/install.sh | bash"#;
    let stack_yaml = r#"apiVersion: stack-cli.dev/v1
kind: StackApp
metadata:
  name: bionic-gpt
  namespace: bionic-gpt
spec:
  components:
    ingress:
      port: 30010
    db:
    rest: {}
    auth:
      hostname-url: http://localhost:30013
      expose_admin: true
    storage:
      install_minio: true
  services:
    web:
      image: ghcr.io/bionic-gpt/bionicgpt:1.11.59
      port: 7703
      migrations_database_url: APP_DATABASE_URL
      init:
        image: ghcr.io/bionic-gpt/bionicgpt-db-migrations:1.11.59
        migrations_database_url: DATABASE_URL
        env:
          - name: INIT_MESSAGE
            value: "warming up"
"#;

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
                class: "mx-auto max-w-6xl px-4 py-16",
                div {
                    class: "grid gap-10 lg:grid-cols-2 lg:items-start",
                    div {
                        class: "space-y-6",
                        h2 {
                            class: "text-2xl font-semibold tracking-tight sm:text-3xl",
                            "One file. One stack. From laptop to production."
                        }
                        p {
                            class: "text-base leading-relaxed",
                            "Stack CLI lets you define your entire web application stack in a single configuration file."
                        }
                        p {
                            class: "text-base leading-relaxed",
                            "If you've used Docker Compose, this should feel familiar. There are no Helm charts to assemble and no manifests to glue together."
                        }
                        p {
                            class: "text-base leading-relaxed",
                            "Run the same configuration locally using Docker Desktop’s built-in Kubernetes, then deploy it unchanged to production. What you develop against locally is what you run in production."
                        }
                    }
                    div {
                        class: "rounded-lg border overflow-hidden",
                        div {
                            class: "bg-gray-100 px-4 py-2 text-sm font-medium",
                            "stack.yaml"
                        }
                        pre {
                            class: "overflow-x-auto bg-gray-50 p-4 text-sm leading-relaxed",
                            code {
                                "{stack_yaml}"
                            }
                        }
                    }
                }
            }

            section {
                class: "mx-auto max-w-6xl px-4 py-16",
                div {
                    class: "grid gap-10 lg:grid-cols-2 lg:items-start",
                    div {
                        class: "space-y-6",
                        h2 {
                            class: "text-2xl font-semibold tracking-tight sm:text-3xl",
                            "The backend you'd build anyway—already wired"
                        }
                        p {
                            class: "text-base leading-relaxed",
                            "Every web application ends up needing the same backend pieces: a database, authentication, APIs, realtime, storage, and ingress."
                        }
                        p {
                            class: "text-base leading-relaxed",
                            "Stack installs and wires these components per application namespace using proven open-source projects—so you don't spend weeks assembling charts, manifests, and glue code."
                        }
                        div {
                            class: "rounded-lg border p-4",
                            p {
                                class: "text-sm leading-relaxed",
                                "This is the setup most teams eventually arrive at—without the experimentation phase."
                            }
                        }
                    }
                    div {
                        class: "rounded-lg border p-6",
                        div {
                            class: "text-sm font-medium",
                            "Included components"
                        }
                        ul {
                            class: "mt-4 space-y-3 text-sm leading-relaxed",
                            li {
                                class: "flex gap-3",
                                span {
                                    class: "mt-2 h-1.5 w-1.5 shrink-0 rounded-full bg-black"
                                }
                                span {
                                    span { class: "font-medium", "PostgreSQL" }
                                    " with app-scoped credentials"
                                }
                            }
                            li {
                                class: "flex gap-3",
                                span {
                                    class: "mt-2 h-1.5 w-1.5 shrink-0 rounded-full bg-black"
                                }
                                span {
                                    span { class: "font-medium", "Auth + OIDC" }
                                    " via Keycloak"
                                }
                            }
                            li {
                                class: "flex gap-3",
                                span {
                                    class: "mt-2 h-1.5 w-1.5 shrink-0 rounded-full bg-black"
                                }
                                span {
                                    span { class: "font-medium", "REST APIs" }
                                    " via PostgREST"
                                }
                            }
                            li {
                                class: "flex gap-3",
                                span {
                                    class: "mt-2 h-1.5 w-1.5 shrink-0 rounded-full bg-black"
                                }
                                span {
                                    span { class: "font-medium", "Realtime" }
                                    " out of the box"
                                }
                            }
                            li {
                                class: "flex gap-3",
                                span {
                                    class: "mt-2 h-1.5 w-1.5 shrink-0 rounded-full bg-black"
                                }
                                span {
                                    span { class: "font-medium", "Object storage" }
                                    " via MinIO"
                                }
                            }
                            li {
                                class: "flex gap-3",
                                span {
                                    class: "mt-2 h-1.5 w-1.5 shrink-0 rounded-full bg-black"
                                }
                                span {
                                    span { class: "font-medium", "Ingress + routing" }
                                    " configured per app"
                                }
                            }
                        }
                        div {
                            class: "mt-6 rounded-lg border p-4",
                            div {
                                class: "text-sm font-medium",
                                "Per app namespace"
                            }
                            p {
                                class: "mt-2 text-sm leading-relaxed",
                                "Each StackApp gets its own isolated set of backend services, so environments stay clean and predictable."
                            }
                        }
                    }
                }
            }

            section {
                class: "mx-auto max-w-6xl px-4 py-16",
                div {
                    class: "grid gap-10 lg:grid-cols-2 lg:items-start",
                    div {
                        class: "space-y-6",
                        h2 {
                            class: "text-2xl font-semibold tracking-tight sm:text-3xl",
                            "Local development that matches production"
                        }
                        p {
                            class: "text-base leading-relaxed",
                            "Stack works with Kubernetes you already have. Docker Desktop includes a built-in Kubernetes cluster, so you just flip it on and start developing."
                        }
                        p {
                            class: "text-base leading-relaxed",
                            "The same StackApp you run locally can be deployed unchanged to staging and production. No separate compose files. No environment-specific rewrites."
                        }
                        div {
                            class: "space-y-3 text-sm leading-relaxed",
                            div {
                                class: "flex gap-3",
                                span {
                                    class: "mt-2 h-1.5 w-1.5 shrink-0 rounded-full bg-black"
                                }
                                span {
                                    span { class: "font-medium", "Zero new tooling:" }
                                    " use Docker Desktop, kubectl, and k9s, the tools you already know."
                                }
                            }
                            div {
                                class: "flex gap-3",
                                span {
                                    class: "mt-2 h-1.5 w-1.5 shrink-0 rounded-full bg-black"
                                }
                                span {
                                    span { class: "font-medium", "Same stack everywhere:" }
                                    " Postgres, auth, REST, realtime, storage, and ingress behave the same locally and in production."
                                }
                            }
                            div {
                                class: "flex gap-3",
                                span {
                                    class: "mt-2 h-1.5 w-1.5 shrink-0 rounded-full bg-black"
                                }
                                span {
                                    span { class: "font-medium", "No platform gap:" }
                                    " what works on your laptop works in the cluster."
                                }
                            }
                        }
                        div {
                            class: "rounded-lg border p-4",
                            p {
                                class: "text-sm leading-relaxed",
                                "Stack doesn't introduce a new runtime or control plane, it builds on standard Kubernetes so local and production stay aligned."
                            }
                        }
                    }
                    figure {
                        class: "rounded-lg border overflow-hidden",
                        div {
                            class: "px-4 py-2 text-sm font-medium",
                            "Using standard Kubernetes tools"
                        }
                        img {
                            class: "block w-full px-4",
                            src: "/landing-page/k9s.png",
                            alt: "k9s showing Stack-managed services running in Docker Desktop Kubernetes"
                        }
                        figcaption {
                            class: "px-4 py-3 text-sm leading-relaxed",
                            "Stack deploys standard Kubernetes resources. You inspect and operate them with kubectl, k9s, and existing tooling."
                        }
                    }
                }
            }

            section {
                class: "mx-auto max-w-6xl px-4 py-16",
                div {
                    class: "grid gap-10 lg:grid-cols-2 lg:items-center",
                    div {
                        class: "space-y-6",
                        h2 {
                            class: "text-2xl font-semibold tracking-tight sm:text-3xl",
                            "Cloud-agnostic by design"
                        }
                        p {
                            class: "text-base leading-relaxed",
                            "Stack is built on standard Kubernetes, so you're not locked into a specific cloud or provider."
                        }
                        p {
                            class: "text-base leading-relaxed",
                            "For production, many developers run Stack on a simple VM using k3s, a lightweight Kubernetes distribution that's easy to set up and operate."
                        }
                        ul {
                            class: "space-y-3 text-sm leading-relaxed",
                            li { "- Run on a single VM with k3s" }
                            li { "- Move between providers without changing your config" }
                            li { "- Lower cost than managed platforms" }
                            li { "- Learn one stack that works everywhere" }
                        }
                        div {
                            class: "rounded-lg border p-4 text-sm leading-relaxed",
                            "Learn it once. Run it anywhere."
                        }
                    }
                    div {
                        class: "rounded-lg border p-6 text-sm text-center",
                        div {
                            class: "mb-2 font-medium",
                            "Same configuration, different environments"
                        }
                        div {
                            img {
                                class: "max-h-full",
                                src: "/landing-page/hosting.svg",
                                alt: "Same configuration diagram"
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
