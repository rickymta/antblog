import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const base = path.dirname(fileURLToPath(import.meta.url));
const checks = [
  ['run', '--project', '11-nullable-patterns', '--', '--verify'],
  ['run', '--project', '12-async-cancellation', '--', '--verify'],
  ['run', '--project', '13-concurrency', '--', '--verify'],
  ['test', '14-unit-test/FareRules.Tests', '--verbosity', 'minimal'],
  ['test', '15-integration-test/QuoteArchive.Tests', '--verbosity', 'minimal'],
  ['test', '20-api-integration-tests', '--verbosity', 'minimal'],
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
console.log('\nPASS: articles 11–20 examples and tests.');
