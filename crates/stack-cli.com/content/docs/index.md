# Stack: Kubernetes + BaaS

Stack is a Kubernetes-first deployment platform with built-in backend services, so you can ship apps and infrastructure together.

- **Kubernetes-first**: runs on any cluster (k3s, k3d, managed, bare metal).
- **Deployment + platform**: one manifest defines app services and platform components.
- **Backend as a service**: Postgres, Auth, REST, Realtime, and Storage are available per namespace.
- **Secrets managed**: Stack generates and wires secrets automatically.
- **Production parity**: the same CRDs and operator flow from dev to prod.

Looking for a deeper dive? Read the [Stack architecture guide](./architecture/) to see how the operator, CRDs, and supporting services interact.

## Other platforms

Other services that solve adjacent parts of the problem:

- [Supabase](https://supabase.com/)
- [Canine](https://docs.canine.sh/)
- [Disco](https://disco.cloud/)
- [Uncloud](https://uncloud.run/)
- [Kamal](https://kamal-deploy.org/)
- [Dokploy](https://dokploy.com/)
