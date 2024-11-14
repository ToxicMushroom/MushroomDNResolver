use anyhow::{Context, Error};
use hickory_resolver::lookup::Lookup;
use hickory_resolver::proto::rr::RecordType;
use hickory_resolver::TokioResolver;
use crate::server::QueryType;

pub(crate) async fn hickory_lookup(resolver: &TokioResolver, x0: &String, query_type: QueryType) -> Result<Lookup, Error> {
    resolver.lookup(x0, match query_type {
        QueryType::UNKNOWN(a) => RecordType::Unknown(a),
        QueryType::A => RecordType::A,
        QueryType::NS => RecordType::NS,
        QueryType::CNAME => RecordType::CNAME,
        QueryType::MX => RecordType::MX,
        QueryType::AAAA => RecordType::AAAA,
    }).await.context("lookup failed")
}