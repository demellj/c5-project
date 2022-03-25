use actix_web::{
    dev::HttpServiceFactory,
    web::{self, ServiceConfig},
    Scope,
};

pub struct RouteBuilder<'a> {
    srv_cfg: &'a mut ServiceConfig,
    scopes: Vec<Scope>,
}

pub trait Router {
    fn build(route_builder: RouteBuilder<'_>) -> RouteBuilder<'_>;
}

impl<'a> RouteBuilder<'a> {
    pub fn new<'b>(srv_cfg: &'b mut ServiceConfig) -> RouteBuilder<'b> {
        RouteBuilder {
            srv_cfg,
            scopes: vec![web::scope("")],
        }
    }

    pub fn mount<F>(mut self, service: F) -> Self
    where
        F: HttpServiceFactory + 'static,
    {
        if let Some(top) = self.scopes.pop() {
            self.scopes.push(top.service(service));
        }
        self
    }

    pub fn extend<R: Router>(mut self, base: &'static str) -> Self {
        self.scopes.push(web::scope(base));
        self = R::build(self);
        let last = self.scopes.pop().unwrap();

        if let Some(top) = self.scopes.pop() {
            self.scopes.push(top.service(last));
        }
        self
    }

    pub fn build(mut self) {
        let top = self.scopes.pop().unwrap();
        self.srv_cfg.service(top);
    }
}
