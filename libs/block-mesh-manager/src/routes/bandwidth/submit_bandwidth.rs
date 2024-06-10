use axum::{Extension, Json};
use http::StatusCode;
use sqlx::PgPool;

use block_mesh_common::interfaces::server_api::{ReportBandwidthRequest, ReportBandwidthResponse};

use crate::database::api_token::find_token::find_token;
use crate::database::bandwidth::create_bandwidth_report::create_bandwidth_report;
use crate::database::user::get_user_by_id::get_user_opt_by_id;
use crate::errors::error::Error;

#[tracing::instrument(name = "submit_bandwidth", skip(pool, body), fields(email = body.email), err, ret)]
pub async fn handler(
    Extension(pool): Extension<PgPool>,
    Json(body): Json<ReportBandwidthRequest>,
) -> Result<Json<ReportBandwidthResponse>, Error> {
    let mut transaction = pool.begin().await.map_err(Error::from)?;
    let api_token = find_token(&mut transaction, &body.api_token)
        .await?
        .ok_or(Error::ApiTokenNotFound)?;
    let user = get_user_opt_by_id(&mut transaction, &api_token.user_id)
        .await?
        .ok_or_else(|| Error::UserNotFound)?;
    if user.email != body.email {
        return Err(Error::UserNotFound);
    }
    create_bandwidth_report(&mut transaction, user.id, body)
        .await
        .map_err(Error::from)?;
    transaction.commit().await.map_err(Error::from)?;

    Ok(Json(ReportBandwidthResponse {
        status_code: u16::from(StatusCode::OK),
    }))
}