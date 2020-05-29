#[macro_use]
extern crate lazy_static;

use hashbrown::HashMap;
use log::{debug, trace};
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde::Deserialize;

static PATH_REWRITE_TARGETS: &'static [u8] = include_bytes!("rewrite_targets.csv");

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Record {
    old: String,
    new: String,
}

lazy_static! {
    static ref MATCH_TARGETS: HashMap<String, String> = {
        let mut map = HashMap::new();
        let mut csv_reader = csv::Reader::from_reader(PATH_REWRITE_TARGETS);
        for path_result in csv_reader.deserialize() {
            let path: Record = path_result.expect("Expected formatted csv");
            map.insert(format!("/{}", path.old), format!("/{}", path.new));
        }
        map
    };
}

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_http_context(|_, _| -> Box<dyn HttpContext> { Box::new(PathRewrite) });
}

struct PathRewrite;

impl Context for PathRewrite {}

impl HttpContext for PathRewrite {
    fn on_http_request_headers(&mut self, _: usize) -> Action {
        if let Some(path) = self.get_http_request_header(":path") {
            trace!("Found path {}", path);
            if let Some(new_target) = MATCH_TARGETS.get(&path) {
                trace!("Found new target, {}", new_target);
                self.set_http_request_header(":path", Some(new_target));
            }
        }
        Action::Continue
    }
}

impl RootContext for PathRewrite {
    fn on_vm_start(&mut self, _: usize) -> bool {
        // call this to intialize on start up
        debug!("Intialized route map with {} entries", MATCH_TARGETS.len());
        true
    }
}
