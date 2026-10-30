use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use hickory_resolver::config::{NameServerConfig, ResolverConfig};
use hickory_resolver::net::runtime::TokioRuntimeProvider;
use hickory_resolver::TokioResolver;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};

// Google public DNS, embedded so lookups do not depend on the OS resolver.
const GOOGLE_DNS: [Ipv4Addr; 2] = [Ipv4Addr::new(8, 8, 8, 8), Ipv4Addr::new(8, 8, 4, 4)];

pub struct GoogleDns {
    resolver: TokioResolver,
}

impl GoogleDns {
    pub fn new() -> Result<Arc<Self>, String> {
        let servers = GOOGLE_DNS.iter().map(|ip| NameServerConfig::udp_and_tcp(IpAddr::V4(*ip))).collect();
        let config = ResolverConfig::from_parts(None, vec![], servers);
        let resolver = TokioResolver::builder_with_config(config, TokioRuntimeProvider::default())
            .build()
            .map_err(|e| format!("Failed to build Google DNS resolver: {e}"))?;
        Ok(Arc::new(Self { resolver }))
    }
}

impl Resolve for GoogleDns {
    fn resolve(&self, name: Name) -> Resolving {
        let resolver = self.resolver.clone();
        Box::pin(async move {
            let lookup = resolver
                .lookup_ip(name.as_str())
                .await
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
            let addrs: Addrs = Box::new(lookup.iter().map(|ip| SocketAddr::new(ip, 0)).collect::<Vec<_>>().into_iter());
            Ok(addrs)
        })
    }
}
