import { callApi, formatDong, newIdempotencyKey, STATUS_LABELS } from './api.js';

const $ = (id) => document.getElementById(id);
const form = $('quote-form');
const state = { quote: null, trip: null, bookingKey: null, cancelKey: null, poller: null };

function user() {
  return `customer:${$('customer-id').value.trim()}`;
}

function showError(message) {
  $('error').textContent = message ?? '';
}

function toggleKindFields() {
  const kind = $('kind').value;
  $('parcel-fields').hidden = kind !== 'parcel';
  $('food-fields').hidden = kind !== 'food';
  $('vehicle').disabled = kind === 'food';
  invalidateQuote();
}

function invalidateQuote() {
  state.quote = null;
  $('book-button').disabled = true;
  $('quote-result').hidden = true;
}

function quoteRequest() {
  const kind = $('kind').value;
  const body = { kind, cityCode: $('city').value, distanceKm: Number($('distance').value), voucherCode: $('voucher').value || null };
  if (kind !== 'food') body.vehicle = $('vehicle').value;
  if (kind === 'parcel') body.parcelWeightKg = Number($('parcel-weight').value);
  if (kind === 'food') body.items = [{ name: 'Đơn món', unitPrice: Number($('food-amount').value), quantity: 1 }];
  return body;
}

form.addEventListener('submit', async (event) => {
  event.preventDefault();
  showError('');
  invalidateQuote();
  try {
    const { quote } = await callApi('POST', '/api/quotes', { user: user(), body: quoteRequest() });
    state.quote = quote;
    state.bookingKey = newIdempotencyKey(); // một khoá cho mỗi báo giá: bấm lại không tạo chuyến trùng
    $('quote-total').textContent = formatDong(quote.total);
    $('quote-peak').textContent = quote.peak ? 'Có' : 'Không';
    $('quote-rain').textContent = quote.raining ? 'Có' : 'Không';
    $('quote-discount').textContent = formatDong(quote.discount);
    $('quote-id').textContent = quote.quoteId;
    $('quote-expires').textContent = new Date(quote.expiresAt).toLocaleTimeString('vi-VN');
    $('quote-result').hidden = false;
    $('book-button').disabled = false;
  } catch (error) {
    showError(error.message);
  }
});

$('book-button').addEventListener('click', async () => {
  if (!state.quote) return;
  showError('');
  $('book-button').disabled = true;
  try {
    const { trip } = await callApi('POST', '/api/trips', {
      user: user(),
      idempotencyKey: state.bookingKey,
      body: { quoteId: state.quote.quoteId, paymentMethod: $('payment-method').value },
    });
    state.cancelKey = newIdempotencyKey();
    renderTrip(trip);
    $('cancel-fee').textContent = '';
    startPolling();
  } catch (error) {
    showError(error.message);
    $('book-button').disabled = false;
  }
});

$('cancel-button').addEventListener('click', async () => {
  if (!state.trip) return;
  showError('');
  try {
    const result = await callApi('POST', `/api/trips/${state.trip.id}/cancel`, {
      user: user(),
      idempotencyKey: state.cancelKey,
      body: {},
    });
    renderTrip(result.trip);
    $('cancel-fee').textContent = `Phí huỷ: ${formatDong(result.cancellationFee)}`;
  } catch (error) {
    showError(error.message);
  }
});

function renderTrip(trip) {
  state.trip = trip;
  $('trip-panel').hidden = false;
  $('trip-id').textContent = trip.id;
  $('trip-status').textContent = STATUS_LABELS[trip.status] ?? trip.status;
  $('trip-status').dataset.status = trip.status;
  $('trip-driver').textContent = trip.driverId ?? '—';
  $('cancel-button').disabled = !['searching', 'accepted', 'arriving'].includes(trip.status);
}

function startPolling() {
  clearInterval(state.poller);
  state.poller = setInterval(async () => {
    if (!state.trip) return;
    try {
      const { trip } = await callApi('GET', `/api/trips/${state.trip.id}`, { user: user() });
      renderTrip(trip);
      if (trip.status === 'completed' || trip.status === 'cancelled') clearInterval(state.poller);
    } catch {
      /* lỗi mạng tạm thời: lần sau thử lại */
    }
  }, 2000);
}

for (const id of ['kind']) $(id).addEventListener('change', toggleKindFields);
for (const id of ['city', 'vehicle', 'distance', 'parcel-weight', 'food-amount', 'voucher', 'customer-id']) {
  $(id).addEventListener('input', invalidateQuote);
}
toggleKindFields();
