import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

if (!process.env.TOI_NOI_PG) {
  console.error('Set TOI_NOI_PG for the dedicated local toi_noi_dotnet_lab database. See article 21 README.');
  process.exit(1);
}
const base = path.dirname(fileURLToPath(import.meta.url));
const checks = [
  ['test', '21-efcore-postgres/Persistence.Tests', '--verbosity', 'minimal'],
  ['test', '22-tracking-projection', '--verbosity', 'minimal'],
  ['test', '23-transactions-idempotency/Booking.Tests', '--verbosity', 'minimal'],
  ['test', '24-background-channel/Worker.Tests', '--verbosity', 'minimal'],
  ['test', '25-cache-resilience/Resilience.Tests', '--verbosity', 'minimal'],
  ['test', '26-outbox-inbox/Outbox.Tests', '--verbosity', 'minimal'],
  ['run', '--project', '27-lifetime-disposal', '-c', 'Release', '--', '--verify'],
  ['run', '--project', '28-benchmark-span', '-c', 'Release', '--', '--verify'],
  ['run', '--project', '29-csharp14', '-c', 'Release', '--', '--verify'],
  ['test', '30-ddd-domain/Domain.Tests', '--verbosity', 'minimal'],
];
for (const args of checks) {
  console.log(`\n> dotnet ${args.join(' ')}`);
  const result = spawnSync('dotnet', args, { cwd: base, stdio: 'inherit', timeout: 180_000 });
  if (result.error) {
    console.error(result.error.message);
    process.exit(1);
  }
  if (result.status !== 0) process.exit(result.status ?? 1);
}
console.log('\nPASS: articles 21–30 examples and real PostgreSQL tests.');
