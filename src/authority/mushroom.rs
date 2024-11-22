use crate::authority::MessageResponseBuilder;
use crate::lookup::hickory_lookup;
use crate::server::{Request, RequestHandler, ResponseHandler, ResponseInfo};
use hickory_proto::op::Header;
use hickory_resolver::TokioResolver;
use std::time::Instant;

pub struct Mushroom {
    pub resolver: TokioResolver,
    pub ipv4_resolver: TokioResolver,
}

#[async_trait::async_trait]
impl RequestHandler for Mushroom {
    async fn handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        mut response_handle: R,
    ) -> ResponseInfo {
        let x = request.request_info().query;
        let now = Instant::now();
        let result = hickory_lookup(self, &x.name().to_string(), x.query_type()).await;
        let lookup_time = now.elapsed().as_millis();

        let mb = MessageResponseBuilder::new(Some(request.raw_query()));

        let response_info = match result {
            Ok(result) => {
                let message_response = mb.build(
                    Header::response_from_request(request.header()),
                    result
                        .record_iter()
                        .filter(|x1| !(x1.record_type().is_soa() || x1.record_type().is_ns())),
                    result.record_iter().filter(|x1| x1.record_type().is_ns()),
                    result.record_iter().filter(|x1| x1.record_type().is_soa()),
                    vec![].into_iter(),
                );
                response_handle
                    .send_response(message_response, lookup_time)
                    .await
                    .expect("being able to send a dns response")
            }
            Err(err) => {
                if err.is_no_records_found() {
                    let message_response =
                        mb.build_no_records(Header::response_from_request(request.header()));
                    response_handle
                        .send_response(message_response, lookup_time)
                        .await
                        .expect("being able to send a dns response")
                } else {
                    // TODO: Return proper error lol
                    let message_response =
                        mb.build_no_records(Header::response_from_request(request.header()));
                    response_handle
                        .send_response(message_response, lookup_time)
                        .await
                        .expect("being able to send a dns response")
                }
            }
        };

        response_info
    }
}
