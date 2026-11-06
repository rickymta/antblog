//! Hợp đồng OpenAPI sinh từ code: DTO, handler và Problem Details cùng một nguồn.

use utoipa::OpenApi;

use crate::dto::{BalanceResponse, EntryResponse, PostEntryRequest, PostingDto};
use crate::handlers;
use crate::problem::{FieldError, Problem};

/// Tài liệu OpenAPI của API `/v1`.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Tới Nơi Ledger API",
        version = "1.0.0",
        description = "API sổ cái double-entry của Tới Nơi. Lỗi trả application/problem+json (RFC 9457)."
    ),
    paths(handlers::post_entry, handlers::get_balance),
    components(schemas(
        PostEntryRequest,
        PostingDto,
        EntryResponse,
        BalanceResponse,
        Problem,
        FieldError
    )),
    tags((name = "ledger", description = "Ghi bút toán và tra số dư"))
)]
pub struct ApiDoc;
