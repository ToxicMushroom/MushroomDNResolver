use hickory_proto::op::Header;
use hickory_resolver::lookup::Lookup;
use hickory_resolver::{ResolveError, TokioResolver};
use crate::authority::{MessageResponse, MessageResponseBuilder};
use crate::lookup::hickory_lookup;
use crate::server::{Request, RequestHandler, ResponseHandler, ResponseInfo};

pub struct Mushroom {
    pub resolver: TokioResolver
}

#[async_trait::async_trait]
impl RequestHandler for Mushroom {
    async fn handle_request<R: ResponseHandler>(&self, request: &Request, mut response_handle: R) -> ResponseInfo {
        let x = request.request_info().query;
        let result = hickory_lookup(&self.resolver, &x.name().to_string(), x.query_type()).await;
        let mb = MessageResponseBuilder::new(Some(request.raw_query()));

        match result {
            Ok(result) => {
                response_handle.send_response(mb.build(
                    Header::response_from_request(request.header()),
                    result.record_iter(),
                    result.record_iter(),
                    result.record_iter(),
                    result.record_iter())
                ).await.expect("TODO: panic message")
            }
            Err(err) => {
                response_handle.send_response(mb.build_no_records(Header::response_from_request(request.header()))).await.expect("success")
            }
        }


    }
}

