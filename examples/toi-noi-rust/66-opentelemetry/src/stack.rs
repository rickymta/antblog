//! Dựng cả phía Rust: server gRPC trên `127.0.0.1:0`, topic Kafka riêng, producer và consumer.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use opentelemetry::KeyValue;
use opentelemetry::metrics::Meter;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

use crate::consumer::EntryConsumer;
use crate::kafka::Lab;
use crate::metrics::Instruments;
use crate::pb::ledger_server::LedgerServer;
use crate::service::{EntryPublisher, LedgerService};

/// Phía Rust đang chạy. Gọi [`Stack::stop`] để tắt theo thứ tự.
pub struct Stack {
    /// Địa chỉ thật của server gRPC (cổng do hệ điều hành cấp).
    pub addr: SocketAddr,
    pub consumer: EntryConsumer,
    pub metrics: Arc<Instruments>,
    pub topic: String,
    lab: Lab,
    shutdown: oneshot::Sender<()>,
    task: JoinHandle<Result<(), tonic::transport::Error>>,
}

impl Stack {
    /// Tạo topic, consumer, producer và bật server.
    ///
    /// # Errors
    /// Thiếu hoặc không nối được Kafka, không bind được cổng.
    pub async fn start(meter: &Meter) -> Result<Stack> {
        let lab = Lab::start().await?;
        let topic = lab.create_topic("entry-posted", 1).await?;
        let group = lab.group_name("projector");
        let metrics = Arc::new(Instruments::new(
            meter,
            vec![
                KeyValue::new("messaging.system.name", "kafka"),
                KeyValue::new("messaging.destination.name", topic.clone()),
                KeyValue::new("messaging.consumer.group.name", group.clone()),
            ],
        ));
        let consumer = EntryConsumer::new(lab.consumer(&group)?, &topic, &group, metrics.clone())?;
        let publisher = EntryPublisher::new(lab.producer()?, topic.clone());
        let service = LedgerService::new(publisher, metrics.clone());

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .context("bind 127.0.0.1:0")?;
        let addr = listener.local_addr()?;
        let (shutdown, rx) = oneshot::channel::<()>();
        let task = tokio::spawn(
            Server::builder()
                .add_service(LedgerServer::new(service))
                .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                    let _ = rx.await;
                }),
        );
        Ok(Stack {
            addr,
            consumer,
            metrics,
            topic,
            lab,
            shutdown,
            task,
        })
    }

    /// Một producer riêng, cùng topic: để kịch bản lô phát thông điệp từ nhiều trace.
    ///
    /// # Errors
    /// Không dựng được producer.
    pub fn new_publisher(&self) -> Result<EntryPublisher> {
        Ok(EntryPublisher::new(
            self.lab.producer()?,
            self.topic.clone(),
        ))
    }

    /// Dừng server (không nhận kết nối mới, chờ request đang chạy), đóng consumer, xoá topic và group.
    /// Chưa tắt telemetry: việc đó làm sau cùng, khi không còn gì tạo span.
    ///
    /// # Errors
    /// Server thoát lỗi hoặc dọn Kafka lỗi.
    pub async fn stop(self) -> Result<()> {
        let _ = self.shutdown.send(());
        self.task.await.context("task server")??;
        drop(self.consumer);
        self.lab.finish().await
    }
}
