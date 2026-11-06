// Mô hình minh họa: không phải bảng giá vận hành thực tế.
decimal distanceKm = 5m;
decimal baseFare = 12000m;
decimal includedKm = 2m;
decimal pricePerExtraKm = 4500m;

decimal extraKm = Math.Max(0m, distanceKm - includedKm);
decimal total = baseFare + extraKm * pricePerExtraKm;

Console.WriteLine($"Quãng đường: {distanceKm} km");
Console.WriteLine($"Cước minh họa: {total:0} VND");
