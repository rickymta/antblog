//! Các lớp bọc handler. Mỗi lớp là một struct generic hiện thực `CommandHandler<C>`
//! bằng cách gọi handler bên trong; thứ tự xếp chồng quyết định thứ tự chạy.

use std::sync::{Arc, Mutex};

use sqlx::PgPool;

use crate::command::{AppError, Command, CommandHandler, Ctx, Validate};
use crate::events::DomainEvent;

/// Lớp validate: từ chối command sai trước khi lớp bên trong chạy.
pub struct Validated<H>(pub H);

impl<C, H> CommandHandler<C> for Validated<H>
where
    C: Command + Validate,
    H: CommandHandler<C>,
{
    async fn handle(&self, cmd: C, ctx: &mut Ctx) -> Result<C::Output, AppError> {
        if let Err(reason) = cmd.validate() {
            ctx.log(format_args!("validate: từ chối {} ({reason})", C::NAME));
            return Err(AppError::Validation(reason));
        }
        ctx.log(format_args!("validate: ok"));
        self.0.handle(cmd, ctx).await
    }
}

/// Lớp trace: ghi vào log lúc vào và lúc ra, kèm kết quả.
pub struct Traced<H>(pub H);

impl<C, H> CommandHandler<C> for Traced<H>
where
    C: Command,
    H: CommandHandler<C>,
{
    async fn handle(&self, cmd: C, ctx: &mut Ctx) -> Result<C::Output, AppError> {
        ctx.log(format_args!("trace: bắt đầu {}", C::NAME));
        let result = self.0.handle(cmd, ctx).await;
        let verdict = if result.is_ok() { "ok" } else { "lỗi" };
        ctx.log(format_args!("trace: kết thúc {} {verdict}", C::NAME));
        result
    }
}

/// Lớp transaction: mở transaction, chạy handler, rồi hoặc (thu sự kiện, ghi outbox, commit)
/// hoặc rollback. Sự kiện chỉ được ghi vào outbox, không phát đi đâu cả.
pub struct Transactional<H> {
    inner: H,
    pool: PgPool,
}

impl<H> Transactional<H> {
    /// Bọc `inner`, mượn kết nối từ `pool`.
    pub fn new(inner: H, pool: PgPool) -> Transactional<H> {
        Transactional { inner, pool }
    }
}

impl<C, H> CommandHandler<C> for Transactional<H>
where
    C: Command,
    H: CommandHandler<C>,
{
    async fn handle(&self, cmd: C, ctx: &mut Ctx) -> Result<C::Output, AppError> {
        let tx = self.pool.begin().await?;
        ctx.install_tx(tx);
        ctx.log(format_args!("tx: begin"));

        let result = self.inner.handle(cmd, ctx).await;
        let mut tx = ctx.remove_tx().ok_or(AppError::NoTransaction)?;

        let output = match result {
            Ok(output) => output,
            Err(err) => {
                // Lỗi ở bất kỳ lớp nào bên trong: hủy hẳn, bỏ sự kiện chưa thu.
                let dropped = ctx.take_events().len();
                tx.rollback().await?;
                ctx.log(format_args!("tx: rollback, bỏ {dropped} sự kiện chưa ghi"));
                return Err(err);
            }
        };

        // Handler xong: thu sự kiện và ghi outbox TRONG cùng transaction.
        let events = ctx.take_events();
        for event in &events {
            sqlx::query("INSERT INTO outbox (topic, key, payload) VALUES ($1, $2, $3::jsonb)")
                .bind(event.topic())
                .bind(event.key())
                .bind(event.payload())
                .execute(&mut *tx)
                .await?;
        }
        ctx.log(format_args!("tx: outbox +{} dòng", events.len()));

        match tx.commit().await {
            Ok(()) => {
                ctx.log(format_args!("tx: commit"));
                Ok(output)
            }
            Err(err) => {
                let state = match &err {
                    sqlx::Error::Database(e) => e.code().map(|c| c.into_owned()),
                    _ => None,
                };
                ctx.log(format_args!(
                    "tx: commit lỗi (SQLSTATE {}), PostgreSQL đã rollback",
                    state.as_deref().unwrap_or("?")
                ));
                Err(AppError::Storage(err))
            }
        }
    }
}

/// Nơi nhận sự kiện của cách làm SAI để đối chứng: phát ngay, không đợi commit.
pub type Sink = Arc<Mutex<Vec<DomainEvent>>>;

/// Lớp phát sự kiện ngay sau handler (đặt BÊN TRONG `Transactional`, tức trước commit).
/// Chỉ để dựng ca đối chứng; không dùng trong pipeline thật.
pub struct NaivePublish<H> {
    inner: H,
    sink: Sink,
}

impl<H> NaivePublish<H> {
    /// Bọc `inner`, phát sự kiện vào `sink` ngay khi handler xong.
    pub fn new(inner: H, sink: Sink) -> NaivePublish<H> {
        NaivePublish { inner, sink }
    }
}

impl<C, H> CommandHandler<C> for NaivePublish<H>
where
    C: Command,
    H: CommandHandler<C>,
{
    async fn handle(&self, cmd: C, ctx: &mut Ctx) -> Result<C::Output, AppError> {
        let output = self.inner.handle(cmd, ctx).await?;
        let events = ctx.take_events();
        ctx.log(format_args!(
            "naive: phát {} sự kiện ra ngoài (commit chưa chạy)",
            events.len()
        ));
        self.sink.lock().expect("mutex").extend(events);
        Ok(output)
    }
}
