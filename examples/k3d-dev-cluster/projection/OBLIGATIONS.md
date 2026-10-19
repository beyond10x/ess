# Obligations: k3d dev cluster, as it ought to be

Every gap this projection did **not** write a change for, and the reason. Applying every file in the tree beside this one leaves exactly these.

Computed from specification `c18c0f5d02b300308071e076508c71541010a362eef3e3842e17b852f8b75e88` against snapshot `cc80082ad1d7daaf379c946dc8bec0461bc664653b2921d70415de5df5f5cdeb` of `k3d-dev-cluster`.

## Decisions owed (16)

### `checkout-exists` — `workloads/shop/deployment/checkout-api`

* **observed:** no deployment named checkout-api in shop
* **why not a patch:** the expectation names the object and not what it would run
* **decision:** write a deployment manifest for shop/checkout-api: this specification says it should exist and says nothing about what it would run

### `shop-probes` — `workloads/shop/deployment/cache-redis`

* **observed:** container redis declares no liveness probe
* **why not a patch:** no value is stated for probes.liveness
* **decision:** choose what makes container `redis` healthy — a path and a port, or a port to connect to — and state it as a `remedy: {probes: …}` on expectation `shop-probes`; probes.liveness unstated

### `shop-probes` — `workloads/shop/deployment/web-server`

* **observed:** container web-server declares no liveness probe
* **why not a patch:** no value is stated for probes.liveness
* **decision:** choose what makes container `web-server` healthy — a path and a port, or a port to connect to — and state it as a `remedy: {probes: …}` on expectation `shop-probes`; probes.liveness unstated

### `shop-probes` — `workloads/shop/deployment/worker-agent`

* **observed:** container agent declares no liveness probe
* **why not a patch:** no value is stated for probes.liveness
* **decision:** choose what makes container `agent` healthy — a path and a port, or a port to connect to — and state it as a `remedy: {probes: …}` on expectation `shop-probes`; probes.liveness unstated

### `shop-probes` — `workloads/shop/statefulset/relay`

* **observed:** container relay declares no liveness probe
* **why not a patch:** no value is stated for probes.liveness
* **decision:** choose what makes container `relay` healthy — a path and a port, or a port to connect to — and state it as a `remedy: {probes: …}` on expectation `shop-probes`; probes.liveness unstated

### `worker-agent-registry` — `workloads/shop/deployment/worker-agent`

* **observed:** container agent pulls registry.example.test/worker-agent from registry.example.test, wanted one of [registry.example.com]
* **why not a patch:** choosing an image is an engineering decision, not a substitution
* **decision:** choose the build of `registry.example.test/worker-agent` that container `agent` should run from one of [registry.example.com]; a rewritten registry prefix is a different image, not the same one somewhere else

### `shop-tags` — `workloads/shop/deployment/worker-agent`

* **observed:** container agent pulls registry.example.test/worker-agent, untagged — which is `latest`
* **why not a patch:** choosing an image is an engineering decision, not a substitution
* **decision:** choose the version of `registry.example.test/worker-agent` that container `agent` should run and write it as a tag or a `sha256:` digest; nothing in the snapshot says which build `registry.example.test/worker-agent` is today

### `shop-tags` — `workloads/shop/statefulset/relay`

* **observed:** container relay pulls registry.example.com/services/relay-8:latest, tagged `latest`
* **why not a patch:** choosing an image is an engineering decision, not a substitution
* **decision:** choose the version of `registry.example.com/services/relay-8:latest` that container `relay` should run and write it as a tag or a `sha256:` digest; nothing in the snapshot says which build `registry.example.com/services/relay-8:latest` is today

### `shop-digests` — `workloads/shop/deployment/cache-redis`

* **observed:** container redis pulls redis:7-alpine, not pinned by digest
* **why not a patch:** choosing an image is an engineering decision, not a substitution
* **decision:** choose the version of `redis:7-alpine` that container `redis` should run and write it as a tag or a `sha256:` digest; nothing in the snapshot says which build `redis:7-alpine` is today

### `shop-digests` — `workloads/shop/deployment/web-server`

* **observed:** container web-server pulls registry.example.test:31721/apps/web-server:AtqQlTV, not pinned by digest
* **why not a patch:** choosing an image is an engineering decision, not a substitution
* **decision:** choose the version of `registry.example.test:31721/apps/web-server:AtqQlTV` that container `web-server` should run and write it as a tag or a `sha256:` digest; nothing in the snapshot says which build `registry.example.test:31721/apps/web-server:AtqQlTV` is today

### `shop-digests` — `workloads/shop/deployment/worker-agent`

* **observed:** container agent pulls registry.example.test/worker-agent, not pinned by digest
* **why not a patch:** choosing an image is an engineering decision, not a substitution
* **decision:** choose the version of `registry.example.test/worker-agent` that container `agent` should run and write it as a tag or a `sha256:` digest; nothing in the snapshot says which build `registry.example.test/worker-agent` is today

### `shop-digests` — `workloads/shop/statefulset/relay`

* **observed:** container relay pulls registry.example.com/services/relay-8:latest, not pinned by digest
* **why not a patch:** choosing an image is an engineering decision, not a substitution
* **decision:** choose the version of `registry.example.com/services/relay-8:latest` that container `relay` should run and write it as a tag or a `sha256:` digest; nothing in the snapshot says which build `registry.example.com/services/relay-8:latest` is today

### `shop-selectors` — `services/shop/lost-lookup`

* **observed:** selector app=retired matches no observed pod
* **why not a patch:** nobody observed what this points at
* **decision:** decide which is true: the workload that should carry app=retired was never deployed, or this selector names labels nothing carries. Deploying one and rewriting the other are different changes and the snapshot does not say which was meant

### `shop-config-refs` — `workloads/shop/deployment/worker-agent`

* **observed:** containers[agent].env[AGENT_TOKEN] requires secret agent-credentials, which was not observed
* **why not a patch:** nobody observed what this points at
* **decision:** create the secret agent-credentials this cluster expects at containers[agent].env[AGENT_TOKEN], or change the reference. Its contents are not in the snapshot — a secret is only ever there as the names of its keys — so nothing here can write one

### `shop-only` — `workloads/kube-system/daemonset/svclb-traefik-2290261f`

* **observed:** sits in kube-system, wanted one of [shop]
* **why not a patch:** `metadata.namespace` cannot be patched on a live object
* **decision:** recreate this workload in one of [shop]: a live object's namespace cannot be patched, so moving it out of `kube-system` is a delete and a create, which is not a patch and not this projection's to write

### `shop-only` — `workloads/kube-system/deployment/coredns`

* **observed:** sits in kube-system, wanted one of [shop]
* **why not a patch:** `metadata.namespace` cannot be patched on a live object
* **decision:** recreate this workload in one of [shop]: a live object's namespace cannot be patched, so moving it out of `kube-system` is a delete and a create, which is not a patch and not this projection's to write

## Refused (0)

None. Every gap this tree does not close is one a person can close.

