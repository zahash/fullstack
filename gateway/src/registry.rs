use std::collections::HashMap;

use matchit::{InsertError, MatchError, Router};

pub struct Registry {
    router: Router<String>,            // Router<ServiceName>
    services: HashMap<String, String>, // HashMap<ServiceName, BaseUrl>
}

pub struct PathResolution<'registry> {
    pub service_name: &'registry str,
    pub base_url: &'registry str,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        service_name: String,
        base_url: String,
        routes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<(), RegistryError> {
        if self.services.contains_key(&service_name) {
            return Err(RegistryError::ServiceNameConflict { with: service_name });
        }

        for route in routes {
            self.router.insert(route, service_name.clone())?;
        }

        self.services.insert(service_name, base_url);
        Ok(())
    }

    pub fn resolve<'registry>(&'registry self, path: &str) -> Option<PathResolution<'registry>> {
        match self.router.at(path) {
            Ok(matched) => {
                let service_name = matched.value.as_str();
                let base_url = self.services.get(service_name)?.as_str();
                Some(PathResolution {
                    service_name,
                    base_url,
                })
            }
            Err(err) => match err {
                MatchError::NotFound => None,
            },
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RegistryError {
    #[error("service name `{with}` already registered")]
    ServiceNameConflict { with: String },

    #[error("error when registering route :: {0}")]
    RouteError(#[from] InsertError),
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            router: Router::new(),
            services: HashMap::new(),
        }
    }
}
