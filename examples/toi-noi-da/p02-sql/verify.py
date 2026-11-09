"""Run the lab and article SQL on an isolated, disposable local PostgreSQL cluster.

Python standard library only. Requires PostgreSQL binaries, no existing server/password.
"""
import argparse
import os
from pathlib import Path
import re
import shutil
import socket
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--pg-bin', type=Path, help='Directory containing initdb, pg_ctl, psql')
    args = parser.parse_args()
    pg_bin = args.pg_bin
    if pg_bin is None:
        found = shutil.which('initdb')
        pg_bin = Path(found).parent if found else Path('C:/Program Files/PostgreSQL/18/bin')
    suffix = '.exe' if os.name == 'nt' else ''
    commands = {name: pg_bin / (name + suffix) for name in ('initdb', 'pg_ctl', 'psql')}
    for command in commands.values():
        if not command.is_file():
            parser.error(f'Missing {command}; supply --pg-bin')
    temp_root = Path(tempfile.gettempdir()).resolve()
    task_dir = Path(tempfile.mkdtemp(prefix='toinoi-da-p02-', dir=temp_root)).resolve()
    # Validate the cleanup target before any recursive removal on Windows.
    if task_dir.parent != temp_root or not task_dir.name.startswith('toinoi-da-p02-'):
        raise RuntimeError('Unexpected temporary workspace')
    creationflags = subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0
    env = os.environ.copy()
    # Do not inherit arbitrary connection/service/encoding settings from the caller.
    for key in list(env):
        if key.startswith('PG'):
            env.pop(key)
    env['PGCLIENTENCODING'] = 'UTF8'

    def run(name, *argv):
        result = subprocess.run([str(commands[name]), *map(str, argv)],
                                capture_output=True, text=True, encoding='utf-8',
                                errors='replace', creationflags=creationflags, env=env)
        if result.returncode:
            raise RuntimeError(f'{name} failed:\n{result.stdout}\n{result.stderr}')
        return result.stdout

    started = False
    stopped = False
    try:
        run('initdb', '-D', task_dir / 'data', '-U', 'da_lab', '--auth=trust',
            '--encoding=UTF8', '--no-locale')
        with socket.socket() as sock:
            sock.bind(('127.0.0.1', 0))
            port = sock.getsockname()[1]
        options = f'-h 127.0.0.1 -p {port}'
        if os.name != 'nt':
            options += " -k ''"
        run('pg_ctl', '-D', task_dir / 'data', '-l', task_dir / 'server.log',
            '-o', options, '-w', 'start')
        started = True
        parts = [(HERE / name).read_text(encoding='utf-8')
                 for name in ('setup.sql', 'queries.sql', 'verify.sql')]
        snippets = 0
        articles = []
        for file in sorted((ROOT / 'api/seed/content').glob('da-*.md')):
            body = file.read_text(encoding='utf-8')
            if not re.search(r'da-0(?:1[1-9]|20)(?:,|\])', body):
                continue
            articles.append(file.name)
            for sql in re.findall(r'```sql\s*\n(.*?)```', body, flags=re.S):
                # Every SQL fence in P02 is executable against setup.sql.
                parts.append(f'\n-- Article: {file.name}\n{sql}')
                snippets += 1
        if len(articles) != 10:
            raise RuntimeError(f'Expected 10 P02 articles, found {len(articles)}')
        script = task_dir / 'check.sql'
        script.write_text('\n'.join(parts), encoding='utf-8')
        output = run('psql', '-X', '-h', '127.0.0.1', '-p', port, '-U', 'da_lab',
                     '-d', 'postgres', '-v', 'ON_ERROR_STOP=1', '-f', script)
        print(output)
        print(f'PASS: lab assertions and {snippets} SQL snippets from {len(articles)} articles')
    finally:
        if started:
            run('pg_ctl', '-D', task_dir / 'data', '-m', 'fast', '-w', 'stop')
            stopped = True
        if not started or stopped:
            shutil.rmtree(task_dir)


if __name__ == '__main__':
    main()
