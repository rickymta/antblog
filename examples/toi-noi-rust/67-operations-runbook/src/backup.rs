//! Sao lưu thật bằng `pg_dump` chạy TRONG container PostgreSQL của lab, rồi restore vào database tạm
//! và so sánh số dư với hash chain. Chỉ dùng `docker exec` vào container hạ tầng lab: không dừng,
//! không khởi động lại, không đụng container nào khác. File dump và database tạm luôn được xoá.
//!
//! Lab KHÔNG dựng WAL archiving nên không có PITR; [`wal_settings`] chỉ đọc cấu hình để nói thật điều đó.

use std::collections::BTreeMap;
use std::str::FromStr;

use anyhow::{Context, Result, bail, ensure};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::{AssertSqlSafe, Row};
use tokio::process::Command;
use uuid::Uuid;

use crate::chain::{Chain, read_chain};

/// Container PostgreSQL của lab; đổi bằng `TOI_NOI_PG_CONTAINER` nếu compose đặt tên khác.
pub fn container_name() -> String {
    std::env::var("TOI_NOI_PG_CONTAINER").unwrap_or_else(|_| "toi-noi-rust-pg".to_string())
}

async fn docker_exec(container: &str, args: &[&str]) -> Result<String> {
    let output = Command::new("docker")
        .arg("exec")
        .arg(container)
        .args(args)
        .output()
        .await
        .context("không chạy được `docker`; Docker đã cài và chạy chưa?")?;
    ensure!(
        output.status.success(),
        "docker exec {container} {} thất bại: {}",
        args.first().copied().unwrap_or(""),
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Dấu vân tay của một bản sổ: dùng so sánh gốc với bản restore.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fingerprint {
    /// Số bút toán.
    pub entries: i64,
    /// Số posting.
    pub postings: i64,
    /// Tổng mọi posting, phải bằng 0.
    pub total_dong: i64,
    /// Số dư projection theo tài khoản.
    pub balances: BTreeMap<String, i64>,
    /// Hash chain.
    pub chain: Chain,
}

/// Tính dấu vân tay của sổ trong schema mà `pool` đang trỏ tới.
///
/// # Errors
/// Lỗi SQL.
pub async fn fingerprint(pool: &PgPool) -> Result<Fingerprint> {
    let entries: i64 = sqlx::query_scalar("SELECT count(*) FROM journal_entries")
        .fetch_one(pool)
        .await?;
    let postings: i64 = sqlx::query_scalar("SELECT count(*) FROM postings")
        .fetch_one(pool)
        .await?;
    let total_dong: i64 =
        sqlx::query_scalar("SELECT COALESCE(sum(amount_dong), 0)::bigint FROM postings")
            .fetch_one(pool)
            .await?;
    let rows =
        sqlx::query("SELECT account_id, balance_dong FROM account_balances ORDER BY account_id")
            .fetch_all(pool)
            .await?;
    Ok(Fingerprint {
        entries,
        postings,
        total_dong,
        balances: rows
            .iter()
            .map(|r| (r.get("account_id"), r.get("balance_dong")))
            .collect(),
        chain: read_chain(pool, None).await?,
    })
}

/// Kết quả một lần sao lưu và restore.
#[derive(Debug, Clone)]
pub struct RestoreReport {
    /// Tên database tạm đã restore vào (đã bị xoá khi hàm trả về).
    pub temp_db: String,
    /// Đường dẫn file dump trong container (đã bị xoá khi hàm trả về).
    pub dump_file: String,
    /// Kích thước file dump, byte (đọc bằng `stat` trong container).
    pub dump_bytes: u64,
    /// Dấu vân tay của bản gốc.
    pub source: Fingerprint,
    /// Dấu vân tay của bản restore.
    pub restored: Fingerprint,
}

/// `pg_dump` schema `schema` ra file tạm trong container, restore vào database tạm, tính dấu vân tay
/// hai bên rồi dọn (database, file) dù thành công hay lỗi.
///
/// # Errors
/// Docker hoặc `pg_dump`/`pg_restore` lỗi, hoặc lỗi SQL khi đọc.
pub async fn dump_and_restore(
    pg_url: &str,
    schema: &str,
    source: &PgPool,
) -> Result<RestoreReport> {
    let options = PgConnectOptions::from_str(pg_url)?;
    let user = options.get_username().to_string();
    let database = options
        .get_database()
        .context("TOI_NOI_PG_URL không có tên database")?
        .to_string();
    let container = container_name();
    let id = Uuid::new_v4().simple().to_string();
    // Cả hai tên đều sinh từ uuid hex nên an toàn để ghép vào SQL và đường dẫn.
    let temp_db = format!("lab67_restore_{id}");
    ensure_lab_database(&temp_db)?;
    let dump_file = format!("/tmp/lab67_{id}.dump");

    let outcome = async {
        docker_exec(
            &container,
            &[
                "pg_dump", "-U", &user, "-d", &database, "-n", schema, "-Fc", "-f", &dump_file,
            ],
        )
        .await?;
        let size = docker_exec(&container, &["stat", "-c", "%s", &dump_file]).await?;
        let dump_bytes: u64 = size.parse().context("kích thước file dump")?;
        ensure!(dump_bytes > 0, "file dump rỗng");

        docker_exec(
            &container,
            &[
                "psql",
                "-U",
                &user,
                "-d",
                "postgres",
                "-v",
                "ON_ERROR_STOP=1",
                "-c",
                &format!("CREATE DATABASE {temp_db}"),
            ],
        )
        .await?;
        docker_exec(
            &container,
            &[
                "pg_restore",
                "-U",
                &user,
                "-d",
                &temp_db,
                "--no-owner",
                "--exit-on-error",
                &dump_file,
            ],
        )
        .await?;

        let restored_options = options
            .clone()
            .database(&temp_db)
            .options([("search_path", schema)]);
        let restored_pool = PgPoolOptions::new()
            .max_connections(2)
            .connect_with(restored_options)
            .await
            .context("không nối được database restore tạm")?;
        let restored = fingerprint(&restored_pool).await;
        restored_pool.close().await;
        Ok::<_, anyhow::Error>(RestoreReport {
            temp_db: temp_db.clone(),
            dump_file: dump_file.clone(),
            dump_bytes,
            source: fingerprint(source).await?,
            restored: restored?,
        })
    }
    .await;

    // Dọn: luôn thử cả hai bước, rồi mới báo lỗi (nếu có).
    let drop_db = docker_exec(
        &container,
        &[
            "psql",
            "-U",
            &user,
            "-d",
            "postgres",
            "-c",
            &format!("DROP DATABASE IF EXISTS {temp_db} WITH (FORCE)"),
        ],
    )
    .await;
    let remove_file = docker_exec(&container, &["rm", "-f", &dump_file]).await;
    let report = outcome?;
    drop_db?;
    remove_file?;
    Ok(report)
}

/// Database `name` còn tồn tại trên server không (dùng để kiểm đã dọn).
///
/// # Errors
/// Lỗi SQL.
pub async fn database_exists(pg_url: &str, name: &str) -> Result<bool> {
    let admin = PgPoolOptions::new()
        .max_connections(1)
        .connect(pg_url)
        .await?;
    let found: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(name)
            .fetch_one(&admin)
            .await?;
    admin.close().await;
    Ok(found)
}

/// File `path` còn trong container PostgreSQL của lab không.
///
/// # Errors
/// Không chạy được `docker`.
pub async fn file_exists_in_container(path: &str) -> Result<bool> {
    let status = Command::new("docker")
        .args(["exec", &container_name(), "test", "-e", path])
        .status()
        .await
        .context("không chạy được `docker`")?;
    Ok(status.success())
}

/// Cấu hình WAL liên quan tới PITR, đọc bằng `SHOW` (chỉ đọc).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalSettings {
    /// `wal_level`.
    pub wal_level: String,
    /// `archive_mode`.
    pub archive_mode: String,
    /// `archive_command` (rỗng khi chưa đặt).
    pub archive_command: String,
}

impl WalSettings {
    /// PITR cần ít nhất `archive_mode` bật và một nơi lưu WAL; lab chỉ kết luận khi chắc chắn.
    pub fn pitr_ready(&self) -> bool {
        self.archive_mode != "off"
            && !self.archive_command.is_empty()
            && self.archive_command != "(disabled)"
    }
}

/// Đọc `wal_level`, `archive_mode`, `archive_command`.
///
/// # Errors
/// Lỗi SQL.
pub async fn wal_settings(pool: &PgPool) -> Result<WalSettings> {
    async fn show(pool: &PgPool, name: &'static str) -> Result<String> {
        // `name` là hằng số của lab, không phải dữ liệu ngoài.
        let value: String = sqlx::query_scalar(AssertSqlSafe(format!("SHOW {name}")))
            .fetch_one(pool)
            .await?;
        Ok(value)
    }
    Ok(WalSettings {
        wal_level: show(pool, "wal_level").await?,
        archive_mode: show(pool, "archive_mode").await?,
        archive_command: show(pool, "archive_command").await?,
    })
}

/// Chặn lỡ tay: tên database tạm phải đúng tiền tố của lab trước khi `DROP`.
pub fn ensure_lab_database(name: &str) -> Result<()> {
    if !name.starts_with("lab67_restore_") {
        bail!("từ chối: {name} không phải database tạm của lab 67");
    }
    Ok(())
}
