use axum_cookie::cookie::{Cookie, cookie::SameSite};

pub struct BaseCookie {
    name: String,
    value: String,
    debug: bool,
    domain: String,
}

impl BaseCookie {
    pub fn new(name: &str, value: &str, debug: bool, domain: &str) -> Self {
        return Self {
            name: name.to_string(),
            value: value.to_string(),
            debug: debug,
            domain: domain.to_string(),
        };
    }
}
impl<'a> Into<Cookie<'a>> for BaseCookie {
    fn into(self) -> Cookie<'a> {
        let domain = if self.debug {
            String::new()
        } else {
            self.domain
        };
        return Cookie::builder(self.name, self.value)
            .http_only(true)
            .secure(!self.debug)
            .same_site(SameSite::Lax)
            .path("/")
            .domain(domain)
            .build();
    }
}
