use serde::Serialize;
use tide::{Body, Response};
use tide::http::StatusCode;
use shared::responses::ApiResponse;

pub trait ApiResponseExt {
    fn to_response(self, status: StatusCode) -> Response;
}

impl<T> ApiResponseExt for ApiResponse<T> where T: Serialize {
    fn to_response(self, status: StatusCode) -> Response
    where T: Serialize {
        let mut resp = Response::new(status);
        resp.set_body(Body::from_json(&self).unwrap());
        resp
    }
}

pub trait BuildApiResponse: Serialize + Sized {
    fn to_response(self, status: StatusCode) -> Response {
        ApiResponseExt::to_response(ApiResponse::new(self),status)
    }
}

impl<T> BuildApiResponse for T where T : Serialize {}
