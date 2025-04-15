use crate::router;
use ic_http_certification::{HttpRequest, HttpResponse};

#[ic_cdk::query]
async fn http_request(
    request: HttpRequest
) -> HttpResponse {
    router::handle(request, true).await
}

#[ic_cdk::update]
async fn http_request_update(
    request: HttpRequest
) -> HttpResponse {
    router::handle(request, false).await
}