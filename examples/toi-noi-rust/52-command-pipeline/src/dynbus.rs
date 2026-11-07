//! Đối chứng: bus kiểu container, tra handler theo `TypeId` lúc chạy.
//! Thiếu handler vẫn biên dịch; lỗi chỉ xuất hiện khi gọi. Mỗi lần gọi tốn một `Box` cho future.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::pin::Pin;

use crate::command::{AppError, Command};

type BoxFuture<T> = Pin<Box<dyn Future<Output = Result<T, AppError>> + Send>>;
type ErasedHandler<C> = Box<dyn Fn(C) -> BoxFuture<<C as Command>::Output> + Send + Sync>;

/// Bus tra cứu lúc chạy. Không dùng `CommandHandler`: `async fn` trong trait không dùng được
/// sau `dyn` (bài 26), nên ở đây handler là closure trả về future đã đóng hộp.
#[derive(Default)]
pub struct DynBus {
    handlers: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl DynBus {
    /// Bus rỗng.
    pub fn new() -> DynBus {
        DynBus::default()
    }

    /// Đăng ký handler cho command `C`. Đăng ký lại thì ghi đè.
    pub fn register<C, F, Fut>(&mut self, handler: F)
    where
        C: Command,
        F: Fn(C) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<C::Output, AppError>> + Send + 'static,
    {
        let erased: ErasedHandler<C> = Box::new(move |cmd| Box::pin(handler(cmd)));
        self.handlers.insert(TypeId::of::<C>(), Box::new(erased));
    }

    /// Gửi command. Chưa đăng ký thì trả `AppError::NoHandler` LÚC CHẠY.
    ///
    /// # Errors
    /// [`AppError::NoHandler`] hoặc lỗi của handler.
    pub async fn send<C: Command>(&self, cmd: C) -> Result<C::Output, AppError> {
        let slot = self
            .handlers
            .get(&TypeId::of::<C>())
            .ok_or(AppError::NoHandler(C::NAME))?;
        let handler = slot
            .downcast_ref::<ErasedHandler<C>>()
            .ok_or(AppError::NoHandler(C::NAME))?;
        handler(cmd).await
    }
}
