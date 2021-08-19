use tide::{Request, Response, Next, Middleware};
use serde_json::json;



#[derive(Debug)]
pub struct ErrResponseToJson;

#[async_trait::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for ErrResponseToJson {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        let mut resp = next.run(req).await;

        if let Some(err) =  resp.error() {
            let status = err.status();
            let body = json!({
                "error": {
                    "status_code" : status.to_string(),
                    "message" : format!("{}", err)
                }
            });
            let mut resp = Response::new(status);
            resp.set_body(body);

            Ok(resp)

        } else {
            let status = resp.status();

            if status.is_success() {
                Ok(resp)
            } else {
                let body = resp.take_body();
                if body.is_empty().expect("no length on response body") {
                    let new_body = json!({
                        "error": {
                            "status_code": status.to_string(),
                            "message": "Something went wrong",
                        }
                    });
                    resp.set_body(new_body);
                } else {
                    resp.set_body(body);
                }
                Ok(resp)
            }
        }
    }
}

// use crate::State;
// use tide::{Request, Response, Next, Middleware};
// use serde_json::json;
// use std::pin::Pin;
// use std::future::Future;



// #[derive(Debug)]
// pub struct ErrResponseToJson;

// impl Middleware<State> for ErrResponseToJson {
//     fn handle<'life0, 'life1, 'async_trait>(
//         &'life0 self,
//         req: Request<State>,
//         next: Next<'life1, State>
//         ) -> Pin<Box<dyn Future<Output = tide::Result> + Send + 'async_trait>> 
//         where 
//             'life0: 'async_trait,
//             'life1: 'async_trait,
//             Self: 'async_trait {
//         Box::pin(async move {
//             let mut resp = next.run(req).await;
 
//             match resp.error() {
//                 None => {
//                     let body = resp.take_body();
//                     if body.is_empty().expect("no length on response body") {
//                         let status = resp.status();
//                         let new_body = json!({
//                             "error": {
//                                 "status_code": status.to_string(),
//                                 "message": "Something went wrong",
//                             }
//                         });
//                         resp.set_body(new_body);
//                     } else {
//                         resp.set_body(body);
//                     }

//                     Ok(resp)
//                 },
//                 Some(err) => {
//                     let status = err.status();
//                     let body = json!({
//                         "error": {
//                             "status_code" : status.to_string(),
//                             "message" : format!("{}", err)
//                         }
//                     });
//                     let mut resp = Response::new(status);
//                     resp.set_body(body);

//                     Ok(resp)
//                 }
//             }

//         })
//     }
// }