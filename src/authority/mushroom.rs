use crate::authority::MessageResponseBuilder;
use crate::lookup::hickory_lookup;
use crate::server::{Request, RequestHandler, ResponseHandler, ResponseInfo};
use hickory_proto::op::Header;
use hickory_proto::rr::RecordType;
use hickory_resolver::TokioResolver;

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
                    result.record_iter().filter(|x1| x1.record_type() != RecordType::SOA && x1.record_type() != RecordType::NS),
                    result.record_iter().filter(|x1| x1.record_type() == RecordType::NS),
                    result.record_iter().filter(|x1| x1.record_type() == RecordType::SOA),
                    vec![].into_iter())
                ).await.expect("TODO: panic message")
            }
            Err(err) => {
                response_handle.send_response(mb.build_no_records(Header::response_from_request(request.header()))).await.expect("success")
            }
        }


    }
}

