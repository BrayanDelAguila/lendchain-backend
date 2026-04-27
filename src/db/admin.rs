use sqlx::types::BigDecimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::loan::Loan;
use crate::models::user::User;

// ── Admin loan queries ────────────────────────────────────────────────────────

pub struct AdminLoanFilters<'a> {
    pub status: Option<&'a str>,
    pub network: Option<&'a str>,
    pub borrower_id: Option<Uuid>,
    pub cursor_id: Option<Uuid>,
    pub limit: i64,
}

pub async fn list_all_loans(
    pool: &PgPool,
    f: AdminLoanFilters<'_>,
) -> Result<Vec<Loan>, sqlx::Error> {
    // Build a dynamic query using QueryBuilder
    let mut qb: sqlx::QueryBuilder<sqlx::Postgres> =
        sqlx::QueryBuilder::new("SELECT * FROM loans WHERE 1=1");

    if let Some(s) = f.status {
        qb.push(" AND status = ").push_bind(s);
    }
    if let Some(n) = f.network {
        qb.push(" AND network = ").push_bind(n);
    }
    if let Some(bid) = f.borrower_id {
        qb.push(" AND borrower_id = ").push_bind(bid);
    }
    if let Some(cursor) = f.cursor_id {
        qb.push(" AND (created_at, id) < (SELECT created_at, id FROM loans WHERE id = ");
        qb.push_bind(cursor);
        qb.push(")");
    }

    qb.push(" ORDER BY created_at DESC, id DESC LIMIT ")
        .push_bind(f.limit);

    qb.build_query_as::<Loan>().fetch_all(pool).await
}

pub async fn update_loan_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
) -> Result<Loan, sqlx::Error> {
    sqlx::query_as::<_, Loan>(
        "UPDATE loans SET status = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
    )
    .bind(status)
    .bind(id)
    .fetch_one(pool)
    .await
}

// ── Admin user queries ────────────────────────────────────────────────────────

pub struct AdminUserFilters<'a> {
    pub kyc_status: Option<&'a str>,
    pub role: Option<&'a str>,
    pub cursor_id: Option<Uuid>,
    pub limit: i64,
}

pub async fn list_all_users(
    pool: &PgPool,
    f: AdminUserFilters<'_>,
) -> Result<Vec<User>, sqlx::Error> {
    let mut qb: sqlx::QueryBuilder<sqlx::Postgres> =
        sqlx::QueryBuilder::new("SELECT * FROM users WHERE 1=1");

    if let Some(k) = f.kyc_status {
        qb.push(" AND kyc_status = ").push_bind(k);
    }
    if let Some(r) = f.role {
        qb.push(" AND role = ").push_bind(r);
    }
    if let Some(cursor) = f.cursor_id {
        qb.push(" AND (created_at, id) < (SELECT created_at, id FROM users WHERE id = ");
        qb.push_bind(cursor);
        qb.push(")");
    }

    qb.push(" ORDER BY created_at DESC, id DESC LIMIT ")
        .push_bind(f.limit);

    qb.build_query_as::<User>().fetch_all(pool).await
}

pub async fn update_user_kyc(
    pool: &PgPool,
    id: Uuid,
    kyc_status: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "UPDATE users SET kyc_status = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
    )
    .bind(kyc_status)
    .bind(id)
    .fetch_one(pool)
    .await
}

// ── Global stats ──────────────────────────────────────────────────────────────

#[derive(Debug, sqlx::FromRow)]
pub struct GlobalStatsRow {
    pub total_users: i64,
    pub total_loans: i64,
    pub pending_loans: i64,
    pub funded_loans: i64,
    pub repaid_loans: i64,
    pub defaulted_loans: i64,
    pub total_volume_usdc: BigDecimal,
    pub total_active_usdc: BigDecimal,
}

pub async fn global_stats(pool: &PgPool) -> Result<GlobalStatsRow, sqlx::Error> {
    sqlx::query_as::<_, GlobalStatsRow>(
        r#"
        SELECT
            (SELECT COUNT(*) FROM users)                                    AS total_users,
            COUNT(*)                                                        AS total_loans,
            COUNT(*) FILTER (WHERE status = 'PENDING')                     AS pending_loans,
            COUNT(*) FILTER (WHERE status = 'FUNDED')                      AS funded_loans,
            COUNT(*) FILTER (WHERE status = 'REPAID')                      AS repaid_loans,
            COUNT(*) FILTER (WHERE status = 'DEFAULTED')                   AS defaulted_loans,
            COALESCE(SUM(amount_usdc), 0)                                  AS total_volume_usdc,
            COALESCE(SUM(amount_usdc) FILTER (WHERE status = 'FUNDED'), 0) AS total_active_usdc
        FROM loans
        "#,
    )
    .fetch_one(pool)
    .await
}
