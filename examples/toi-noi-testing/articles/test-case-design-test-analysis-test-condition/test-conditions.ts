/**
 * TCD-01 — Kết quả test analysis cho voucher (SPEC R-VOUCHER-01…08, R-FARE-06, R-FARE-07, R-FOOD-05),
 * viết thành DỮ LIỆU có kiểu để test kiểm được: truy vết, mức ưu tiên, câu hỏi gửi người viết đặc tả.
 * Không có giá trị cụ thể ở đây — chọn giá trị là việc của test design (TCD-02 trở đi).
 */

/** Mã yêu cầu trong SPEC.md mà bài phân tích. */
export type RCode = `R-${'VOUCHER' | 'FARE' | 'FOOD' | 'GEN' | 'QUOTE'}-${string}`;

export type Priority = 'cao' | 'trung-binh' | 'thap';

export interface TestCondition {
  /** TCOND-VOUCHER-nn */
  id: string;
  /** Một câu khẳng định kiểm chứng được: «cái gì» chứ chưa phải «kiểm thế nào». */
  statement: string;
  /** Phần tử của test basis mà condition suy ra từ đó (≥ 1). */
  basis: RCode[];
  /** Ưu tiên sơ bộ theo rủi ro (mục 4.5 của bài). */
  priority: Priority;
  /** Rủi ro sản phẩm mà condition giúp giảm. */
  risk: string;
  /** Condition đang chờ câu trả lời của người viết đặc tả (Q-xx) thì chưa thiết kế test case được. */
  blockedBy?: string;
}

export const TEST_CONDITIONS: TestCondition[] = [
  // R-VOUCHER-02, -03 — nhận diện mã
  { id: 'TCOND-VOUCHER-01', statement: 'Mã có khoảng trắng hai đầu hoặc chữ thường được chuẩn hoá (bỏ khoảng trắng, viết hoa) rồi mới tra', basis: ['R-VOUCHER-02'], priority: 'trung-binh', risk: 'khách gõ đúng mã mà bị báo không tồn tại ⇒ bỏ đặt chuyến' },
  { id: 'TCOND-VOUCHER-02', statement: 'Mã rỗng hoặc chỉ gồm khoảng trắng được coi như không nhập voucher', basis: ['R-VOUCHER-02'], priority: 'thap', risk: 'ô voucher để trống làm hỏng báo giá' },
  { id: 'TCOND-VOUCHER-03', statement: 'Mã không có trong danh sách voucher bị từ chối với VOUCHER_NOT_FOUND', basis: ['R-VOUCHER-03'], priority: 'trung-binh', risk: 'mã bịa vẫn được giảm giá' },

  // R-VOUCHER-01, -04 — thời hạn
  { id: 'TCOND-VOUCHER-04', statement: 'Ngày đặt nằm trong [validFrom; validTo], tính cả ngày đầu và ngày cuối, được chấp nhận', basis: ['R-VOUCHER-01', 'R-VOUCHER-04'], priority: 'trung-binh', risk: 'khách bị từ chối đúng ngày cuối chương trình' },
  { id: 'TCOND-VOUCHER-05', statement: 'Ngày đặt trước validFrom hoặc sau validTo bị từ chối với VOUCHER_EXPIRED', basis: ['R-VOUCHER-04'], priority: 'trung-binh', risk: 'voucher hết hạn vẫn giảm giá ⇒ thất thu' },
  { id: 'TCOND-VOUCHER-06', statement: 'Ngày đặt tính theo giờ Việt Nam, không theo múi giờ của máy chủ', basis: ['R-VOUCHER-04', 'R-GEN-02'], priority: 'cao', risk: 'lệch 7 giờ quanh nửa đêm: voucher mở sớm hoặc đóng muộn trên máy chủ chạy UTC' },

  // R-VOUCHER-01, -05 — phạm vi
  { id: 'TCOND-VOUCHER-07', statement: 'Dịch vụ không thuộc kinds của voucher bị từ chối với VOUCHER_NOT_APPLICABLE', basis: ['R-VOUCHER-05'], priority: 'trung-binh', risk: 'voucher đồ ăn bị dùng cho đặt xe' },
  { id: 'TCOND-VOUCHER-08', statement: 'Thành phố không thuộc cityCodes của voucher bị từ chối với VOUCHER_NOT_APPLICABLE', basis: ['R-VOUCHER-05'], priority: 'trung-binh', risk: 'voucher Hà Nội bị dùng ở TP. Hồ Chí Minh' },
  { id: 'TCOND-VOUCHER-09', statement: 'kinds rỗng nghĩa là mọi dịch vụ; cityCodes rỗng nghĩa là mọi thành phố', basis: ['R-VOUCHER-01', 'R-VOUCHER-05'], priority: 'trung-binh', risk: 'danh sách rỗng bị hiểu là «không áp dụng ở đâu»' },

  // R-VOUCHER-06, R-FARE-06, R-FOOD-05, R-FARE-07 — tiền giảm
  { id: 'TCOND-VOUCHER-10', statement: 'Số tiền áp voucher A nhỏ hơn minAmount bị từ chối với VOUCHER_MIN_AMOUNT; A bằng minAmount là hợp lệ', basis: ['R-VOUCHER-06'], priority: 'cao', risk: 'sai phép so sánh ở biên ⇒ từ chối khách hợp lệ hoặc giảm giá cho đơn không đủ điều kiện' },
  { id: 'TCOND-VOUCHER-11', statement: 'Khi chưa chạm trần, tiền giảm bằng percent% × A', basis: ['R-VOUCHER-06'], priority: 'cao', risk: 'giảm sai tiền trên mọi chuyến dùng voucher' },
  { id: 'TCOND-VOUCHER-12', statement: 'Tiền giảm không vượt maxDiscount', basis: ['R-VOUCHER-06'], priority: 'cao', risk: 'chuyến dài được giảm không giới hạn ⇒ thất thu lớn' },
  { id: 'TCOND-VOUCHER-13', statement: 'Với đặt xe và bưu kiện, A là số tiền sau bước 3 (đã gồm hệ số cao điểm và phụ phí mưa)', basis: ['R-FARE-06', 'R-VOUCHER-06'], priority: 'cao', risk: 'tính giảm trên cước cơ bản ⇒ lệch giá giữa báo giá và đối soát' },
  { id: 'TCOND-VOUCHER-14', statement: 'Với đồ ăn, A là tiền món: minAmount so với tiền món và voucher không trừ vào phí giao', basis: ['R-FOOD-05', 'R-VOUCHER-06'], priority: 'cao', risk: 'giảm cả phí giao hoặc cho đơn món nhỏ cộng phí giao vượt ngưỡng' },
  { id: 'TCOND-VOUCHER-15', statement: 'Voucher không kéo cước xuống dưới cước tối thiểu của loại xe', basis: ['R-FARE-07', 'R-FARE-06'], priority: 'cao', risk: 'chuyến ngắn gần như miễn phí' },

  // R-VOUCHER-07 — lượt dùng
  { id: 'TCOND-VOUCHER-16', statement: 'Báo giá không tiêu lượt; tạo chuyến từ báo giá có voucher tiêu một lượt', basis: ['R-VOUCHER-07'], priority: 'cao', risk: 'khách báo giá nhiều lần bị mất lượt, hoặc đặt chuyến mà không mất lượt' },
  { id: 'TCOND-VOUCHER-17', statement: 'Đã dùng đủ perCustomerPerDay lượt trong ngày thì bị từ chối VOUCHER_USAGE_LIMIT, ở cả lúc báo giá lẫn lúc tạo chuyến', basis: ['R-VOUCHER-07', 'R-QUOTE-03'], priority: 'cao', risk: 'giữ sẵn nhiều báo giá để dùng một voucher nhiều lần' },
  { id: 'TCOND-VOUCHER-18', statement: 'Chuyến bị huỷ trả lại lượt dùng', basis: ['R-VOUCHER-07'], priority: 'trung-binh', risk: 'khách huỷ chuyến mất luôn voucher ⇒ khiếu nại' },
  { id: 'TCOND-VOUCHER-19', statement: 'Lượt dùng đếm theo ngày giờ Việt Nam; sang ngày mới thì có lượt mới', basis: ['R-VOUCHER-07', 'R-GEN-02'], priority: 'trung-binh', risk: 'ngày mới bắt đầu lúc 07:00 giờ VN trên máy chủ UTC' },
  { id: 'TCOND-VOUCHER-20', statement: 'perCustomerPerDay lớn hơn 1 cho phép dùng đúng chừng đó lượt trong ngày', basis: ['R-VOUCHER-01', 'R-VOUCHER-07'], priority: 'trung-binh', risk: 'giới hạn bị cài cứng bằng 1' },
  { id: 'TCOND-VOUCHER-21', statement: 'Thời hạn và phạm vi chỉ kiểm khi báo giá, không kiểm lại khi tạo chuyến từ báo giá còn hiệu lực', basis: ['R-VOUCHER-07', 'R-QUOTE-04'], priority: 'trung-binh', risk: 'khách báo giá lúc 23:58 ngày cuối bị từ chối lúc đặt' },

  // R-VOUCHER-08 — thứ tự lỗi
  { id: 'TCOND-VOUCHER-22', statement: 'Vi phạm nhiều quy tắc voucher thì chỉ báo lỗi đứng trước theo thứ tự của R-VOUCHER-08', basis: ['R-VOUCHER-08'], priority: 'thap', risk: 'thông báo lỗi gây hiểu nhầm (sai lỗi được hiển thị)' },

  // R-VOUCHER-01 — thuộc tính dữ liệu, chưa kiểm được qua test object này
  { id: 'TCOND-VOUCHER-23', statement: 'Voucher có percent ngoài 1–50 không được áp dụng', basis: ['R-VOUCHER-01'], priority: 'thap', risk: 'cấu hình nhầm 500% làm cước âm', blockedBy: 'Q-04' },
];

export interface SpecQuestion {
  id: string;
  about: RCode[];
  question: string;
  /** Vì sao đây là defect tiềm ẩn của test basis (mơ hồ, thiếu, mâu thuẫn). */
  kind: 'mơ hồ' | 'thiếu' | 'cần xác nhận';
}

/** Câu hỏi phát sinh trong lúc phân tích — testware của test analysis (CTFL 1.4.3). */
export const SPEC_QUESTIONS: SpecQuestion[] = [
  { id: 'Q-01', about: ['R-FARE-07', 'R-FARE-10', 'R-VOUCHER-06'], kind: 'mơ hồ', question: 'Khi cước tối thiểu nâng giá lên, «tiền giảm» hiển thị cho khách là tiền giảm tính theo R-VOUCHER-06 hay phần thực sự được giảm?' },
  { id: 'Q-02', about: ['R-VOUCHER-02'], kind: 'cần xác nhận', question: 'Khoảng trắng ở giữa mã («GIAM 20») có được bỏ không, hay chỉ bỏ hai đầu?' },
  { id: 'Q-03', about: ['R-VOUCHER-07'], kind: 'cần xác nhận', question: 'Chuyến khách huỷ có phí (charged) vẫn trả lại lượt voucher? Có giới hạn số lần huỷ để lấy lại lượt không?' },
  { id: 'Q-04', about: ['R-VOUCHER-01'], kind: 'thiếu', question: 'Voucher có percent ngoài 1–50 hoặc maxDiscount = 0 thì hệ thống xử lý thế nào, ai kiểm (công cụ quản trị không có trong lab), mã lỗi gì?' },
  { id: 'Q-05', about: ['R-VOUCHER-07'], kind: 'mơ hồ', question: '«Cùng ngày» của chuyến đã tạo tính theo thời điểm tạo chuyến (requestedAt) hay thời điểm báo giá của chuyến đó?' },
];
