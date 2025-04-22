pub struct Configuration {
    pub sqlite_path: String,
    pub roam_path: String,
    pub ip_addr: String,
    pub port: u16,
}

impl Configuration {
    pub fn get_url(&self, protocol: bool) -> String {
        if protocol {
            format!("http://{}:{}", self.ip_addr, self.port)
        } else {
            format!("{}:{}", self.ip_addr, self.port)
        }
    }
}
