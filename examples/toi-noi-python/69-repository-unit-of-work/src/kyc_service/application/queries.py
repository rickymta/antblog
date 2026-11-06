"""Query: nhu cầu đọc. Trả view, không trả aggregate."""

from dataclasses import dataclass

from kyc_service.application.commands import ApplicationId
from kyc_service.application.messages import Query
from kyc_service.application.views import ApplicationView


@dataclass(frozen=True)
class GetApplication(Query[ApplicationView]):
    application_id: ApplicationId


@dataclass(frozen=True)
class ListReviewQueue(Query[tuple[ApplicationView, ...]]):
    pass
