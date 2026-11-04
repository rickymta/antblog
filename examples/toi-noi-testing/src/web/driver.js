import { callApi, formatDong, newIdempotencyKey, STATUS_LABELS } from './api.js';

const $ = (id) => document.getElementById(id);
let currentTrip = null;

function user() {
  return `driver:${$('driver-id').value.trim()}`;
}

function showError(message) {
  $('error').textContent = message ?? '';
}

async function loadAvailable() {
  showError('');
  const list = $('available-trips');
  list.replaceChildren();
  try {
    const { trips } = await callApi('GET', '/api/driver/available-trips', { user: user() });
    if (trips.length === 0) {
      list.append(Object.assign(document.createElement('li'), { textContent: 'Chưa có chuyến nào.' }));
    }
    for (const trip of trips) {
      const item = document.createElement('li');
      item.dataset.testid = `available-${trip.id}`;
      item.textContent = `${trip.kind} · ${trip.distanceKm} km · ${formatDong(trip.fare)} `;
      const button = Object.assign(document.createElement('button'), { type: 'button', textContent: 'Nhận chuyến' });
      button.dataset.testid = `accept-${trip.id}`;
      button.addEventListener('click', () => accept(trip.id));
      item.append(button);
      list.append(item);
    }
  } catch (error) {
    showError(error.message);
  }
}

async function accept(tripId) {
  showError('');
  try {
    const { trip } = await callApi('POST', `/api/trips/${tripId}/accept`, { user: user(), body: { etaMinutes: Number($('eta').value) } });
    render(trip);
    await loadAvailable();
  } catch (error) {
    showError(error.message);
  }
}

async function act(action) {
  if (!currentTrip) return;
  showError('');
  try {
    const options = { user: user(), body: {} };
    if (action === 'cancel') options.idempotencyKey = newIdempotencyKey();
    const { trip } = await callApi('POST', `/api/trips/${currentTrip.id}/${action}`, options);
    render(action === 'cancel' ? null : trip);
  } catch (error) {
    showError(error.message);
  }
}

function render(trip) {
  currentTrip = trip;
  $('current-trip').hidden = trip === null;
  if (!trip) return;
  $('current-trip-id').textContent = trip.id;
  $('driver-trip-status').textContent = STATUS_LABELS[trip.status] ?? trip.status;
  $('driver-trip-status').dataset.status = trip.status;
}

$('load-available').addEventListener('click', loadAvailable);
document.querySelectorAll('[data-action]').forEach((button) => {
  button.addEventListener('click', () => act(button.dataset.action));
});
