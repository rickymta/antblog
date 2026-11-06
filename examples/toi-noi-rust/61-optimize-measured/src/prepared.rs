//! Prepared statement của sqlx: cache theo kết nối, đếm bằng message `Parse` ở tầng dây
//! (và `pg_prepared_statements` của phiên). `pg_stat_statements` không dùng được trên server lab.

use sqlx::{AssertSqlSafe, PgConnection};

use crate::db::LabDb;
use crate::proxy::{Proxy, Wire};

/// Kết quả một thí nghiệm.
#[derive(Debug, Clone)]
pub struct Report {
    /// Mô tả.
    pub label: String,
    /// Số lần thực thi câu lệnh.
    pub executions: u64,
    /// Message đã đi qua proxy trong lúc chạy.
    pub wire: Wire,
    /// Số dòng của `pg_prepared_statements` (cache phía server của phiên) sau khi chạy.
    pub prepared_on_server: i64,
}

async fn prepared_on_server(conn: &mut PgConnection) -> Result<i64, sqlx::Error> {
    // Chạy không prepare (`persistent(false)`) để chính câu đếm này không hiện trong view.
    sqlx::query_scalar("select count(*) from pg_prepared_statements")
        .persistent(false)
        .fetch_one(conn)
        .await
}

async fn report(
    label: &str,
    executions: u64,
    proxy: &Proxy,
    before: Wire,
    conn: &mut PgConnection,
) -> Result<Report, sqlx::Error> {
    let wire = proxy.stats.snapshot() - before;
    Ok(Report {
        label: label.to_string(),
        executions,
        wire,
        prepared_on_server: prepared_on_server(conn).await?,
    })
}

/// Cùng một câu SQL `times` lần trên một kết nối, `persistent` bật hay tắt cache.
/// Hai chế độ dùng hai câu SQL khác nhau (`+ 1` và `+ 2`): cache của sqlx tra theo nội dung SQL
/// trước, nên một câu đã có trong cache thì `persistent(false)` cũng không còn tác dụng.
///
/// # Errors
/// Lỗi SQL.
pub async fn same_text(
    conn: &mut PgConnection,
    proxy: &Proxy,
    times: i64,
    persistent: bool,
) -> Result<Report, sqlx::Error> {
    let before = proxy.stats.snapshot();
    let (sql, delta) = if persistent {
        ("select $1::bigint + 1", 1)
    } else {
        ("select $1::bigint + 2", 2)
    };
    for i in 0..times {
        let v: i64 = sqlx::query_scalar(sql)
            .persistent(persistent)
            .bind(i)
            .fetch_one(&mut *conn)
            .await?;
        assert_eq!(v, i + delta);
    }
    let label = if persistent {
        "cùng câu SQL, cache bật"
    } else {
        "cùng câu SQL, persistent(false)"
    };
    report(label, times.unsigned_abs(), proxy, before, conn).await
}

/// SQL động `where id in ($1, ..., $n)` với n = 1..=`max`: mỗi n là một câu SQL khác.
///
/// # Errors
/// Lỗi SQL.
pub async fn in_lists(
    conn: &mut PgConnection,
    proxy: &Proxy,
    max: usize,
) -> Result<Report, sqlx::Error> {
    let before = proxy.stats.snapshot();
    for n in 1..=max {
        let params: Vec<String> = (1..=n).map(|i| format!("${i}")).collect();
        // Chỉ ghép số thứ tự tham số, không có đầu vào ngoài.
        let sql = format!(
            "select count(*) from accounts where id in ({})",
            params.join(", ")
        );
        let mut q = sqlx::query_scalar::<_, i64>(AssertSqlSafe(sql));
        for _ in 0..n {
            q = q.bind("platform:revenue");
        }
        let found = q.fetch_one(&mut *conn).await?;
        assert_eq!(found, 1);
    }
    report("IN ($1..$n), n = 1..max", max as u64, proxy, before, conn).await
}

/// Một câu `where id = any($1)` với mảng dài 1..=`max`: một câu SQL cho mọi kích thước.
///
/// # Errors
/// Lỗi SQL.
pub async fn any_arrays(
    conn: &mut PgConnection,
    proxy: &Proxy,
    max: usize,
) -> Result<Report, sqlx::Error> {
    let before = proxy.stats.snapshot();
    for n in 1..=max {
        let ids = vec!["platform:revenue"; n];
        let found: i64 = sqlx::query_scalar("select count(*) from accounts where id = any($1)")
            .bind(&ids[..])
            .fetch_one(&mut *conn)
            .await?;
        assert_eq!(found, 1);
    }
    report(
        "= any($1), mảng dài 1..max",
        max as u64,
        proxy,
        before,
        conn,
    )
    .await
}

/// Kết nối mới (cache rỗng) chạy cùng câu SQL: server phải `Parse` lại.
///
/// # Errors
/// Lỗi SQL hoặc kết nối.
pub async fn fresh_connection(
    db: &LabDb,
    proxy: &Proxy,
    times: i64,
) -> Result<Report, sqlx::Error> {
    use sqlx::Connection;
    let mut conn = db.connect_via(proxy).await?;
    let r = same_text(&mut conn, proxy, times, true).await?;
    conn.close().await?;
    Ok(r)
}

/// Cache nhỏ (`capacity`) với `distinct` câu SQL khác nhau: cache chặn ở sức chứa, câu cũ bị đóng.
///
/// # Errors
/// Lỗi SQL hoặc kết nối.
pub async fn small_cache(
    db: &LabDb,
    proxy: &Proxy,
    capacity: usize,
    distinct: usize,
) -> Result<Report, sqlx::Error> {
    use sqlx::Connection;
    let mut conn = db.connect_via_with_cache(proxy, capacity).await?;
    let mut r = in_lists(&mut conn, proxy, distinct).await?;
    r.label = format!("cache {capacity} chỗ, {distinct} câu SQL khác nhau");
    conn.close().await?;
    Ok(r)
}
