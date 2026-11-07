//! Ca thử nghiệm PostgreSQL cô lập: schema `lab_<uuid>` và năm role mang cùng tiền tố.
//!
//! Role là đối tượng của cả cluster (không nằm trong schema), nên tên role phải có tiền tố theo
//! uuid của ca và phải được dọn cuối ca. Role không có `LOGIN`: ta đổi vai bằng `SET ROLE` trên
//! kết nối quản trị, nên lab không cần mật khẩu nào.

use std::future::Future;

use anyhow::{Context, Result, bail};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::{AssertSqlSafe, Connection, Executor, PgConnection};
use uuid::Uuid;

use crate::chain::ChainTable;

const SCHEMA_SQL: &str = include_str!("../sql/01_schema.sql");
const APPEND_ONLY_SQL: &str = include_str!("../sql/02_append_only.sql");
const GRANTS_SQL: &str = include_str!("../sql/03_grants.sql");

/// Các vai trò của lab. `Owner` là role chạy migration và sở hữu bảng; `Writer` là danh tính của
/// service ledger lúc chạy thật; ba role còn lại ứng với Customer Support, Finance, Admin (§11.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Chủ bảng (role migration).
    Owner,
    /// Service ledger.
    Writer,
    /// Customer Support.
    Cs,
    /// Finance.
    Finance,
    /// Admin.
    Admin,
}

impl Role {
    /// Mọi role, `Owner` đứng đầu.
    pub const ALL: [Role; 5] = [
        Role::Owner,
        Role::Writer,
        Role::Cs,
        Role::Finance,
        Role::Admin,
    ];

    /// Hậu tố trong tên role, ví dụ `writer`.
    pub fn suffix(self) -> &'static str {
        match self {
            Role::Owner => "owner",
            Role::Writer => "writer",
            Role::Cs => "cs",
            Role::Finance => "finance",
            Role::Admin => "admin",
        }
    }
}

/// Một ca thử: `app` chạy dưới role `Writer`, `admin` là kết nối superuser của lab.
#[derive(Clone)]
pub struct Lab {
    /// Pool của ứng dụng: mọi kết nối đã `SET ROLE` về `Writer` và `search_path` về schema của ca.
    pub app: PgPool,
    /// Pool quản trị (superuser của database lab), cũng đã đặt `search_path`.
    pub admin: PgPool,
    /// Tên schema `lab_<uuid>`; cũng là tiền tố tên role.
    pub schema: String,
    url: String,
}

fn pg_url() -> Result<String> {
    pg_url_from(std::env::var("TOI_NOI_PG_URL").ok())
}

/// Đọc URL từ giá trị biến môi trường. Thiếu hoặc rỗng là lỗi, không phải lý do để bỏ qua ca thử.
///
/// # Errors
/// Khi `value` là `None` hoặc chỉ có khoảng trắng.
pub fn pg_url_from(value: Option<String>) -> Result<String> {
    match value {
        Some(v) if !v.trim().is_empty() => Ok(v),
        _ => bail!(
            "thiếu TOI_NOI_PG_URL. Bật hạ tầng: docker compose -f \
             examples/toi-noi-rust/infra/docker-compose.yml up -d --wait, \
             rồi export TOI_NOI_PG_URL theo examples/toi-noi-rust/infra/README.md"
        ),
    }
}

async fn run(pool: &PgPool, sql: String) -> Result<()> {
    pool.execute(AssertSqlSafe(sql)).await?;
    Ok(())
}

impl Lab {
    /// Tên đầy đủ của một role trong ca này.
    pub fn role_name(&self, role: Role) -> String {
        format!("{}_{}", self.schema, role.suffix())
    }

    /// Tạo role, schema (chủ là `Owner`), chạy ba file SQL và nạp danh mục tài khoản.
    /// Lỗi giữa chừng thì dọn những gì đã tạo rồi mới trả lỗi.
    ///
    /// # Errors
    /// Thiếu `TOI_NOI_PG_URL`, không kết nối được, hoặc một câu SQL lỗi.
    pub async fn new() -> Result<Lab> {
        let url = pg_url()?;
        // Tên do ta sinh (hex): an toàn để ghép vào SQL.
        let schema = format!("lab_{}", Uuid::new_v4().simple());
        let sp = format!("SET search_path TO {schema}");
        let admin = PgPoolOptions::new()
            .max_connections(3)
            .after_connect(move |conn, _| {
                let sql = sp.clone();
                Box::pin(async move {
                    conn.execute(AssertSqlSafe(sql)).await?;
                    Ok(())
                })
            })
            .connect(&url)
            .await
            .context("không kết nối được PostgreSQL ở TOI_NOI_PG_URL (compose đã bật chưa?)")?;

        let setup = Lab::setup(&admin, &schema).await;
        if let Err(e) = setup {
            let _ = Lab::drop_all(&admin, &schema).await;
            admin.close().await;
            return Err(e.context("dựng ca thử thất bại, đã dọn phần đã tạo"));
        }

        let app_init = format!("SET search_path TO {schema}; SET ROLE {schema}_writer");
        let app = PgPoolOptions::new()
            .max_connections(8)
            .after_connect(move |conn, _| {
                let sql = app_init.clone();
                Box::pin(async move {
                    conn.execute(AssertSqlSafe(sql)).await?;
                    Ok(())
                })
            })
            .connect(&url)
            .await;
        match app {
            Ok(app) => Ok(Lab {
                app,
                admin,
                schema,
                url,
            }),
            Err(e) => {
                let _ = Lab::drop_all(&admin, &schema).await;
                admin.close().await;
                Err(anyhow::Error::new(e).context("dựng ca thử thất bại, đã dọn phần đã tạo"))
            }
        }
    }

    async fn setup(admin: &PgPool, p: &str) -> Result<()> {
        for role in Role::ALL {
            run(admin, format!("CREATE ROLE {p}_{} NOLOGIN", role.suffix())).await?;
        }
        run(admin, format!("CREATE SCHEMA {p} AUTHORIZATION {p}_owner")).await?;

        // DDL chạy dưới role Owner: bảng thuộc Owner, không thuộc superuser, không thuộc Writer.
        // detach: kết nối riêng, bị đóng khi drop, nên role Owner không rò về pool dù lỗi giữa chừng.
        let mut conn = admin.acquire().await?.detach();
        conn.execute(AssertSqlSafe(format!("SET ROLE {p}_owner")))
            .await?;
        sqlx::raw_sql(SCHEMA_SQL).execute(&mut conn).await?;
        sqlx::raw_sql(APPEND_ONLY_SQL).execute(&mut conn).await?;
        sqlx::raw_sql(AssertSqlSafe(GRANTS_SQL.replace("{p}", p)))
            .execute(&mut conn)
            .await?;
        for (id, kind) in [
            ("customer:CU-001", "liability"),
            ("customer:CU-002", "liability"),
            ("driver:DR-001", "liability"),
            ("driver:DR-002", "liability"),
            ("platform:revenue", "revenue"),
            ("psp:vixanh:clearing", "asset"),
        ] {
            sqlx::query("INSERT INTO accounts (id, kind) VALUES ($1, $2)")
                .bind(id)
                .bind(kind)
                .execute(&mut conn)
                .await?;
        }
        conn.close().await?;
        Ok(())
    }

    // Xoá schema rồi từng role. Chạy được cả khi chỉ tạo dở. Một bước lỗi không dừng các bước sau,
    // vì role sót lại là rác của cả cluster; trả lỗi đầu tiên sau khi đã thử hết.
    async fn drop_all(admin: &PgPool, p: &str) -> Result<()> {
        let mut first = run(admin, format!("DROP SCHEMA IF EXISTS {p} CASCADE"))
            .await
            .err();
        for role in Role::ALL {
            let name = format!("{p}_{}", role.suffix());
            if let Err(e) = Lab::drop_role(admin, &name).await {
                first.get_or_insert(e);
            }
        }
        first.map_or(Ok(()), Err)
    }

    async fn drop_role(admin: &PgPool, name: &str) -> Result<()> {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = $1)")
                .bind(name)
                .fetch_one(admin)
                .await?;
        if exists {
            // Gỡ mọi quyền còn sót trong database này rồi xoá role.
            run(admin, format!("DROP OWNED BY {name}")).await?;
            run(admin, format!("DROP ROLE {name}")).await?;
        }
        Ok(())
    }

    /// Kết nối riêng đã đổi sang `role`, `search_path` về schema của ca. Mỗi lần gọi là một kết
    /// nối mới nên không rò vai sang kết nối khác.
    ///
    /// # Errors
    /// Không kết nối được hoặc `SET ROLE` lỗi.
    pub async fn conn_as(&self, role: Role) -> Result<PgConnection> {
        let opts: PgConnectOptions = self.url.parse()?;
        let mut conn = PgConnection::connect_with(&opts).await?;
        conn.execute(AssertSqlSafe(format!(
            "SET search_path TO {}; SET ROLE {}",
            self.schema,
            self.role_name(role)
        )))
        .await?;
        Ok(conn)
    }

    /// Tạo bảng `entry_chain_naive` (không khoá, không ràng buộc) cho thí nghiệm rẽ nhánh.
    ///
    /// # Errors
    /// Lỗi SQL.
    pub async fn create_naive_chain(&self) -> Result<()> {
        let mut conn = self.conn_as(Role::Owner).await?;
        let t = ChainTable::Naive.name();
        conn.execute(AssertSqlSafe(format!(
            "CREATE TABLE {t} (LIKE entry_chain); GRANT SELECT, INSERT ON {t} TO {w}",
            w = self.role_name(Role::Writer)
        )))
        .await?;
        conn.close().await?;
        Ok(())
    }

    /// Đóng pool, xoá schema và mọi role của ca.
    ///
    /// # Errors
    /// Lỗi khi xoá.
    pub async fn cleanup(self) -> Result<()> {
        self.app.close().await;
        Lab::drop_all(&self.admin, &self.schema).await?;
        self.admin.close().await;
        Ok(())
    }
}

/// Chạy `body` trong một ca, luôn dọn schema và role, kể cả khi `body` panic.
///
/// # Errors
/// Lỗi tạo hoặc dọn ca, hoặc lỗi `body` trả về.
pub async fn with_lab<F, Fut>(body: F) -> Result<()>
where
    F: FnOnce(Lab) -> Fut,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    let lab = Lab::new().await?;
    let joined = tokio::spawn(body(lab.clone())).await;
    lab.cleanup().await?;
    match joined {
        Ok(result) => result,
        Err(join_err) => std::panic::resume_unwind(join_err.into_panic()),
    }
}

/// Như [`with_lab`] nhưng trả giá trị `body` tạo ra (sau khi đã dọn ca).
///
/// # Errors
/// Lỗi tạo hoặc dọn ca, hoặc lỗi `body` trả về.
pub async fn with_lab_value<F, Fut, T>(body: F) -> Result<T>
where
    F: FnOnce(Lab) -> Fut + Send + 'static,
    Fut: Future<Output = Result<T>> + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = tokio::sync::oneshot::channel();
    with_lab(async move |lab| {
        let value = body(lab).await?;
        let _ = tx.send(value);
        Ok(())
    })
    .await?;
    Ok(rx.await?)
}

/// Số role còn mang tiền tố `lab_<uuid>` của một ca (kỳ vọng 0 sau khi dọn).
///
/// # Errors
/// Không kết nối được hoặc lỗi SQL.
pub async fn leftover_roles(schema: &str) -> Result<i64> {
    let mut conn = PgConnection::connect(&pg_url()?).await?;
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_roles WHERE rolname LIKE $1")
        .bind(format!("{schema}\\_%"))
        .fetch_one(&mut conn)
        .await?;
    conn.close().await?;
    Ok(n)
}

/// Schema có còn tồn tại không.
///
/// # Errors
/// Không kết nối được hoặc lỗi SQL.
pub async fn schema_exists(schema: &str) -> Result<bool> {
    let mut conn = PgConnection::connect(&pg_url()?).await?;
    let found: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM information_schema.schemata WHERE schema_name = $1)",
    )
    .bind(schema)
    .fetch_one(&mut conn)
    .await?;
    conn.close().await?;
    Ok(found)
}
