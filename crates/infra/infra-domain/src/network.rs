//! Services, ingresses, and Gateway API gateways and HTTP routes: how traffic reaches workloads.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::code::{InfraCode, ValidationErrors};
use crate::observation::{identity, port_string, string_map, Identity};
use crate::raw::{RawGateway, RawHttpRoute, RawIngress, RawIngressBackend, RawService};

/// A service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Service {
    /// Identity.
    pub identity: Identity,
    /// Labels.
    pub labels: BTreeMap<String, String>,
    /// The service type; `ClusterIP` when the document omitted it, which is the API's default.
    pub service_type: String,
    /// The pod selector. Empty means the service selects nothing by label — endpoints are
    /// managed by hand or by a controller, which is legal and carried as-is.
    pub selector: BTreeMap<String, String>,
    /// The declared ports, in declared order.
    pub ports: Vec<ServicePort>,
}

/// One declared service port.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServicePort {
    /// The port's name, when declared.
    pub name: Option<String>,
    /// The exposed port.
    pub port: u16,
    /// The pod-side port: a number or a named container port, rendered as text. Equal to
    /// `port` when the document omitted it, which is the API's default.
    pub target_port: String,
    /// The protocol; `TCP` when the document omitted it, which is the API's default.
    pub protocol: String,
}

/// An ingress: rules from hosts and paths to backend services.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Ingress {
    /// Identity.
    pub identity: Identity,
    /// Labels.
    pub labels: BTreeMap<String, String>,
    /// The routing rules, in declared order.
    pub rules: Vec<IngressRule>,
    /// The backend used when no rule matches.
    pub default_backend: Option<IngressBackend>,
}

/// One ingress rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IngressRule {
    /// The host the rule applies to, when declared.
    pub host: Option<String>,
    /// The paths, in declared order.
    pub paths: Vec<IngressPath>,
}

/// One routed path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IngressPath {
    /// The URL path.
    pub path: Option<String>,
    /// How the path matches, such as `Prefix`.
    pub path_type: Option<String>,
    /// Where matching traffic goes.
    pub backend: IngressBackend,
}

/// A backend service reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IngressBackend {
    /// The service's name, in the ingress's own namespace.
    pub service: String,
    /// The service port, by name or number, rendered as text.
    pub port: Option<String>,
}

impl Service {
    pub(crate) fn from_raw(
        raw: &RawService,
        location: &str,
        errors: &mut ValidationErrors,
    ) -> Option<Self> {
        let identity = identity(&raw.metadata, true, location, errors)?;
        let mut ports = Vec::with_capacity(raw.spec.ports.len());
        for (index, port) in raw.spec.ports.iter().enumerate() {
            let port_location = format!("{location}.spec.ports[{index}]");
            let Some(number) = port.port else {
                errors.refuse(
                    InfraCode::MalformedObject,
                    format!("{port_location}.port"),
                    "a service port without a port number exposes nothing",
                );
                continue;
            };
            let Ok(number) = u16::try_from(number) else {
                errors.refuse(
                    InfraCode::MalformedObject,
                    format!("{port_location}.port"),
                    format!("{number} is not a port; ports are 1–65535"),
                );
                continue;
            };
            let target_port = match &port.target_port {
                Some(value) => {
                    match port_string(value, &format!("{port_location}.targetPort"), errors) {
                        Some(target) => target,
                        None => continue,
                    }
                }
                None => number.to_string(),
            };
            ports.push(ServicePort {
                name: port.name.clone(),
                port: number,
                target_port,
                protocol: port.protocol.clone().unwrap_or_else(|| "TCP".to_owned()),
            });
        }
        Some(Self {
            identity,
            labels: string_map(
                &raw.metadata.labels,
                &format!("{location}.metadata.labels"),
                errors,
            ),
            service_type: raw
                .spec
                .service_type
                .clone()
                .unwrap_or_else(|| "ClusterIP".to_owned()),
            selector: string_map(
                &raw.spec.selector,
                &format!("{location}.spec.selector"),
                errors,
            ),
            ports,
        })
    }
}

impl IngressBackend {
    /// Validates one backend, refusing a backend that names no service.
    fn from_raw(
        raw: &RawIngressBackend,
        location: &str,
        errors: &mut ValidationErrors,
    ) -> Option<Self> {
        let Some(service) = raw
            .service
            .as_ref()
            .and_then(|service| service.name.clone())
            .filter(|name| !name.is_empty())
        else {
            errors.refuse(
                InfraCode::IncompleteBackend,
                location.to_owned(),
                "the backend names no service; traffic routed nowhere is a rule that means nothing",
            );
            return None;
        };
        let port = raw.service.as_ref().and_then(|backend| {
            backend.port.as_ref().and_then(|port| {
                port.name
                    .clone()
                    .or_else(|| port.number.map(|number| number.to_string()))
            })
        });
        Some(Self { service, port })
    }
}

impl Ingress {
    pub(crate) fn from_raw(
        raw: &RawIngress,
        location: &str,
        errors: &mut ValidationErrors,
    ) -> Option<Self> {
        let identity = identity(&raw.metadata, true, location, errors)?;
        let mut rules = Vec::with_capacity(raw.spec.rules.len());
        for (rule_index, rule) in raw.spec.rules.iter().enumerate() {
            let mut paths = Vec::new();
            if let Some(http) = &rule.http {
                for (path_index, path) in http.paths.iter().enumerate() {
                    let path_location = format!(
                        "{location}.spec.rules[{rule_index}].http.paths[{path_index}].backend"
                    );
                    let backend = if let Some(backend) = &path.backend {
                        IngressBackend::from_raw(backend, &path_location, errors)
                    } else {
                        errors.refuse(
                            InfraCode::IncompleteBackend,
                            path_location,
                            "the path declares no backend at all",
                        );
                        None
                    };
                    if let Some(backend) = backend {
                        paths.push(IngressPath {
                            path: path.path.clone(),
                            path_type: path.path_type.clone(),
                            backend,
                        });
                    }
                }
            }
            rules.push(IngressRule {
                host: rule.host.clone(),
                paths,
            });
        }
        let default_backend = raw.spec.default_backend.as_ref().and_then(|backend| {
            IngressBackend::from_raw(backend, &format!("{location}.spec.defaultBackend"), errors)
        });
        Some(Self {
            identity,
            labels: string_map(
                &raw.metadata.labels,
                &format!("{location}.metadata.labels"),
                errors,
            ),
            rules,
            default_backend,
        })
    }
}

/// The Gateway API group, which a parent reference names when it omits its group.
pub const GATEWAY_API_GROUP: &str = "gateway.networking.k8s.io";

/// A Gateway API gateway: a class, its listeners and the addresses it was assigned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gateway {
    /// Identity.
    pub identity: Identity,
    /// Labels.
    pub labels: BTreeMap<String, String>,
    /// The gateway class that implements it.
    pub gateway_class: String,
    /// The listeners, in declared order.
    pub listeners: Vec<GatewayListener>,
    /// The addresses the implementation assigned, in the order the status lists them.
    ///
    /// Assigned rather than declared, and kept anyway: an address a gateway was given is the
    /// name the cluster's HTTP edge is reached at, which is what a map of that edge is for.
    pub addresses: Vec<GatewayAddress>,
}

/// One gateway listener.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GatewayListener {
    /// The listener's name.
    pub name: String,
    /// The hostname the listener accepts, when it restricts one.
    pub hostname: Option<String>,
    /// The port.
    pub port: u16,
    /// The protocol, such as `HTTPS`.
    pub protocol: String,
}

/// One assigned gateway address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GatewayAddress {
    /// The address type; `IPAddress` when the status omitted it, which is the API's default.
    pub address_type: String,
    /// The address.
    pub value: String,
}

/// A Gateway API HTTP route: hostnames, the parents it attaches to, and rules to backends.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HttpRoute {
    /// Identity.
    pub identity: Identity,
    /// Labels.
    pub labels: BTreeMap<String, String>,
    /// The hostnames the route answers for, in declared order.
    pub hostnames: Vec<String>,
    /// The parents it attaches to, in declared order.
    pub parents: Vec<RouteParentRef>,
    /// The rules, in declared order.
    pub rules: Vec<HttpRouteRule>,
}

/// One parent reference of a route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteParentRef {
    /// The parent's API group; [`GATEWAY_API_GROUP`] when the document omitted it.
    pub group: String,
    /// The parent's kind; `Gateway` when the document omitted it.
    pub kind: String,
    /// The parent's name.
    pub name: String,
    /// The parent's namespace, when it names one; the route's own otherwise.
    pub namespace: Option<String>,
    /// The parent's listener the route attaches to, when it names one.
    pub section_name: Option<String>,
}

/// One route rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HttpRouteRule {
    /// The request matches, in declared order.
    pub matches: Vec<HttpRouteMatch>,
    /// The backends, in declared order.
    pub backends: Vec<RouteBackendRef>,
}

/// One request match; only the path is modelled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HttpRouteMatch {
    /// The path match, when the match declares one.
    pub path: Option<HttpPathMatch>,
}

/// A path match.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HttpPathMatch {
    /// How the path matches; `PathPrefix` when the document omitted it, the API's default.
    pub match_type: String,
    /// The path; `/` when the document omitted it, the API's default.
    pub value: String,
}

/// One backend reference of a route rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteBackendRef {
    /// The backend's API group; the core group `""` when the document omitted it.
    pub group: String,
    /// The backend's kind; `Service` when the document omitted it.
    pub kind: String,
    /// The backend's name.
    pub name: String,
    /// The backend's namespace, when it names one; the route's own otherwise.
    pub namespace: Option<String>,
    /// The backend port.
    pub port: Option<u16>,
    /// The share of traffic the backend receives.
    pub weight: Option<u32>,
}

/// A required, non-empty string field, refused with its location when absent.
fn required(
    value: Option<&String>,
    location: String,
    what: &str,
    errors: &mut ValidationErrors,
) -> Option<String> {
    let text = value.filter(|text| !text.is_empty()).cloned();
    if text.is_none() {
        errors.refuse(
            InfraCode::MalformedObject,
            location,
            format!("{what} is missing or empty"),
        );
    }
    text
}

/// A port number, refused with its location when outside 1–65535.
fn port_number(number: i64, location: String, errors: &mut ValidationErrors) -> Option<u16> {
    match u16::try_from(number) {
        Ok(port) if port > 0 => Some(port),
        _ => {
            errors.refuse(
                InfraCode::MalformedObject,
                location,
                format!("{number} is not a port; ports are 1–65535"),
            );
            None
        }
    }
}

impl Gateway {
    pub(crate) fn from_raw(
        raw: &RawGateway,
        location: &str,
        errors: &mut ValidationErrors,
    ) -> Option<Self> {
        let identity = identity(&raw.metadata, true, location, errors)?;
        let gateway_class = required(
            raw.spec.gateway_class_name.as_ref(),
            format!("{location}.spec.gatewayClassName"),
            "the gateway class",
            errors,
        );
        let mut listeners = Vec::with_capacity(raw.spec.listeners.len());
        for (index, listener) in raw.spec.listeners.iter().enumerate() {
            let at = format!("{location}.spec.listeners[{index}]");
            let name = required(
                listener.name.as_ref(),
                format!("{at}.name"),
                "a listener's name",
                errors,
            );
            let protocol = required(
                listener.protocol.as_ref(),
                format!("{at}.protocol"),
                "a listener's protocol",
                errors,
            );
            let port = if let Some(number) = listener.port {
                port_number(number, format!("{at}.port"), errors)
            } else {
                errors.refuse(
                    InfraCode::MalformedObject,
                    format!("{at}.port"),
                    "a listener without a port accepts nothing",
                );
                None
            };
            if let (Some(name), Some(protocol), Some(port)) = (name, protocol, port) {
                listeners.push(GatewayListener {
                    name,
                    hostname: listener.hostname.clone(),
                    port,
                    protocol,
                });
            }
        }
        let mut addresses = Vec::with_capacity(raw.status.addresses.len());
        for (index, address) in raw.status.addresses.iter().enumerate() {
            if let Some(value) = required(
                address.value.as_ref(),
                format!("{location}.status.addresses[{index}].value"),
                "an address's value",
                errors,
            ) {
                addresses.push(GatewayAddress {
                    address_type: address
                        .address_type
                        .clone()
                        .unwrap_or_else(|| "IPAddress".to_owned()),
                    value,
                });
            }
        }
        Some(Self {
            identity,
            labels: string_map(
                &raw.metadata.labels,
                &format!("{location}.metadata.labels"),
                errors,
            ),
            gateway_class: gateway_class?,
            listeners,
            addresses,
        })
    }
}

impl HttpRoute {
    pub(crate) fn from_raw(
        raw: &RawHttpRoute,
        location: &str,
        errors: &mut ValidationErrors,
    ) -> Option<Self> {
        let identity = identity(&raw.metadata, true, location, errors)?;
        let mut parents = Vec::with_capacity(raw.spec.parent_refs.len());
        for (index, parent) in raw.spec.parent_refs.iter().enumerate() {
            if let Some(name) = required(
                parent.name.as_ref(),
                format!("{location}.spec.parentRefs[{index}].name"),
                "a parent reference's name",
                errors,
            ) {
                parents.push(RouteParentRef {
                    group: parent
                        .group
                        .clone()
                        .unwrap_or_else(|| GATEWAY_API_GROUP.to_owned()),
                    kind: parent.kind.clone().unwrap_or_else(|| "Gateway".to_owned()),
                    name,
                    namespace: parent.namespace.clone(),
                    section_name: parent.section_name.clone(),
                });
            }
        }
        let mut rules = Vec::with_capacity(raw.spec.rules.len());
        for (rule_index, rule) in raw.spec.rules.iter().enumerate() {
            let matches = rule
                .matches
                .iter()
                .map(|entry| HttpRouteMatch {
                    path: entry.path.as_ref().map(|path| HttpPathMatch {
                        match_type: path
                            .match_type
                            .clone()
                            .unwrap_or_else(|| "PathPrefix".to_owned()),
                        value: path.value.clone().unwrap_or_else(|| "/".to_owned()),
                    }),
                })
                .collect();
            let mut backends = Vec::with_capacity(rule.backend_refs.len());
            for (index, backend) in rule.backend_refs.iter().enumerate() {
                let at = format!("{location}.spec.rules[{rule_index}].backendRefs[{index}]");
                let Some(name) = backend.name.clone().filter(|name| !name.is_empty()) else {
                    errors.refuse(
                        InfraCode::IncompleteBackend,
                        format!("{at}.name"),
                        "the backend names nothing; traffic routed nowhere is a rule that means nothing",
                    );
                    continue;
                };
                let port = match backend.port {
                    Some(number) => match port_number(number, format!("{at}.port"), errors) {
                        Some(port) => Some(port),
                        None => continue,
                    },
                    None => None,
                };
                let weight = match backend.weight {
                    Some(number) => {
                        if let Ok(weight) = u32::try_from(number) {
                            Some(weight)
                        } else {
                            errors.refuse(
                                InfraCode::MalformedObject,
                                format!("{at}.weight"),
                                format!("{number} is not a weight; weights are 0 or more"),
                            );
                            continue;
                        }
                    }
                    None => None,
                };
                backends.push(RouteBackendRef {
                    group: backend.group.clone().unwrap_or_default(),
                    kind: backend.kind.clone().unwrap_or_else(|| "Service".to_owned()),
                    name,
                    namespace: backend.namespace.clone(),
                    port,
                    weight,
                });
            }
            rules.push(HttpRouteRule { matches, backends });
        }
        Some(Self {
            identity,
            labels: string_map(
                &raw.metadata.labels,
                &format!("{location}.metadata.labels"),
                errors,
            ),
            hostnames: raw.spec.hostnames.clone(),
            parents,
            rules,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_route_fills_the_api_defaults_and_keeps_a_parent_of_another_kind_as_declared() {
        let raw: RawHttpRoute = serde_json::from_value(serde_json::json!({
            "metadata": { "name": "shop-api", "namespace": "shop", "uid": "r1" },
            "spec": {
                "hostnames": ["shop.example.internal"],
                "parentRefs": [
                    { "name": "edge", "namespace": "gateways", "sectionName": "https" },
                    { "group": "gateway.networking.x-k8s.io", "kind": "ListenerSet", "name": "extra" }
                ],
                "rules": [ {
                    "matches": [ { "path": {} } ],
                    "backendRefs": [ { "name": "shop-api", "port": 8080, "weight": 1 } ]
                } ]
            }
        }))
        .expect("the raw route parses");
        let mut errors = ValidationErrors::new();
        let route = HttpRoute::from_raw(&raw, "r", &mut errors).expect("identity is intact");
        assert!(errors.is_empty(), "nothing to refuse: {errors}");
        assert_eq!(route.parents[0].group, GATEWAY_API_GROUP);
        assert_eq!(route.parents[0].kind, "Gateway");
        assert_eq!(route.parents[1].kind, "ListenerSet");
        let backend = &route.rules[0].backends[0];
        assert_eq!(
            (backend.group.as_str(), backend.kind.as_str()),
            ("", "Service")
        );
        assert_eq!(backend.port, Some(8080));
        let path = route.rules[0].matches[0]
            .path
            .as_ref()
            .expect("a path match");
        assert_eq!(
            (path.match_type.as_str(), path.value.as_str()),
            ("PathPrefix", "/")
        );
    }

    #[test]
    fn a_route_backend_without_a_name_and_a_listener_without_a_port_are_refused() {
        let route: RawHttpRoute = serde_json::from_value(serde_json::json!({
            "metadata": { "name": "shop-api", "namespace": "shop", "uid": "r1" },
            "spec": { "rules": [ { "backendRefs": [ { "port": 80 } ] } ] }
        }))
        .expect("the raw route parses");
        let mut errors = ValidationErrors::new();
        HttpRoute::from_raw(&route, "r", &mut errors);
        assert!(errors.contains(InfraCode::IncompleteBackend), "{errors}");

        let gateway: RawGateway = serde_json::from_value(serde_json::json!({
            "metadata": { "name": "edge", "namespace": "gateways", "uid": "g1" },
            "spec": { "gatewayClassName": "shared", "listeners": [ { "name": "https", "protocol": "HTTPS" } ] }
        }))
        .expect("the raw gateway parses");
        let mut errors = ValidationErrors::new();
        let gateway = Gateway::from_raw(&gateway, "g", &mut errors).expect("identity is intact");
        assert!(errors.contains(InfraCode::MalformedObject), "{errors}");
        assert_eq!(gateway.listeners, Vec::<GatewayListener>::new());
    }

    #[test]
    fn a_service_port_out_of_range_is_refused_and_the_rest_of_the_service_survives_the_pass() {
        let raw: RawService = serde_json::from_value(serde_json::json!({
            "metadata": { "name": "web", "namespace": "app", "uid": "u1" },
            "spec": { "ports": [
                { "port": 70000 },
                { "port": 443, "targetPort": "https" }
            ] }
        }))
        .expect("the raw service parses");
        let mut errors = ValidationErrors::new();
        let service = Service::from_raw(&raw, "s", &mut errors).expect("identity is intact");
        assert!(
            errors.contains(InfraCode::MalformedObject),
            "70000 is refused: {errors}"
        );
        assert_eq!(service.ports.len(), 1, "the valid port is still collected");
        assert_eq!(service.ports[0].target_port, "https");
    }

    #[test]
    fn an_ingress_path_without_a_service_name_is_refused_as_an_incomplete_backend() {
        let raw: RawIngress = serde_json::from_value(serde_json::json!({
            "metadata": { "name": "edge", "namespace": "app", "uid": "u1" },
            "spec": { "rules": [ { "host": "x.test", "http": { "paths": [
                { "path": "/", "backend": { "service": {} } }
            ] } } ] }
        }))
        .expect("the raw ingress parses");
        let mut errors = ValidationErrors::new();
        Ingress::from_raw(&raw, "i", &mut errors);
        assert!(
            errors.contains(InfraCode::IncompleteBackend),
            "expected INFRA-INGRESS-001, got: {errors}"
        );
    }

    #[test]
    fn defaults_fill_in_what_the_api_would_have_cluster_ip_and_tcp_and_port_equal_target() {
        let raw: RawService = serde_json::from_value(serde_json::json!({
            "metadata": { "name": "web", "namespace": "app", "uid": "u1" },
            "spec": { "ports": [ { "port": 80 } ] }
        }))
        .expect("the raw service parses");
        let mut errors = ValidationErrors::new();
        let service = Service::from_raw(&raw, "s", &mut errors).expect("valid");
        assert!(errors.is_empty(), "nothing to refuse: {errors}");
        assert_eq!(service.service_type, "ClusterIP");
        assert_eq!(service.ports[0].protocol, "TCP");
        assert_eq!(service.ports[0].target_port, "80");
    }
}
