"""Thông điệp gửi qua mediator. `R` là kiểu kết quả mà người gửi nhận lại."""


class Message[R]:
    """Gốc chung. Chỉ để gắn kiểu kết quả vào thông điệp; không có hành vi."""


class Command[R](Message[R]):
    """Ý định làm thay đổi trạng thái. Chạy trong transaction."""


class Query[R](Message[R]):
    """Nhu cầu đọc. Không đổi trạng thái, không mở transaction."""
