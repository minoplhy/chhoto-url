use std::env;

#[derive(Clone)]
pub struct Config {
    pub db_url: String,
    pub port: u16,
    pub cache_control_header: Option<String>,
    pub api_url: String,
    pub public_mode: bool,
    pub site_url: String,
    pub redirect_method: String,
    pub password: Option<String>,
    pub slug_style: String,
    pub slug_length: usize,
    pub api_key_size: usize,
}

impl Config {
    pub fn build() -> Self {
        let db_location = env::var("db_url")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(String::from("urls.sqlite"));

        let port = env::var("port")
            .unwrap_or(String::from("4567"))
            .parse::<u16>()
            .expect("Supplied port is not an integer");

        let cache_control_header = env::var("cache_control_header")
            .ok()
            .filter(|s| !s.trim().is_empty());

        let api_url = {
            let mut get_api_url = env::var("api_url".replace("//", "/"))
            .ok()
            .unwrap_or_default();
            if get_api_url.ends_with("/") {
                get_api_url.pop();
            }
            get_api_url
        };

        let public_mode = env::var("public_mode") == Ok(String::from("Enable"));
        
        let site_url = env::var("site_url")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(String::from("unset"));

        let redirect_method = env::var("redirect_method").unwrap_or(String::from("PERMANENT"));

        let password = env::var("password")
            .ok();

        let slug_style = env::var("slug_style").unwrap_or(String::from("Pair"));
        let mut slug_length = env::var("slug_length")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(8);
        if slug_length < 4 {
            slug_length = 4;
        }

        let api_key_size = env::var("api_key_size")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(32);
    
        Self {
            db_url: (db_location), 
            port: (port), 
            cache_control_header: (cache_control_header), 
            api_url: (api_url),
            public_mode: (public_mode),
            site_url: (site_url),
            redirect_method: (redirect_method),
            password: (password),
            slug_style: (slug_style),
            slug_length: (slug_length),
            api_key_size: (api_key_size)
        }
    }
}