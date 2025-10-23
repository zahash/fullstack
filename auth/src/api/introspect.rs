// TODO

pub const PATH: &str = "/introspect";

/*
 Request
 {
     "permissions": [
         "/sysinfo",
         "/access-token/generate"
     ]
 }

 Response
 {
     "permissions": {
         "/sysinfo": false,
         "/access-token/generate": true
     }
 }
*/

pub struct RequestBody {
    pub permissions: Vec<String>,
}
