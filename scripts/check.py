#!/usr/bin/env python3
"""Run the same native validation locally and in CI without updating baselines."""
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]

def main():
    cargo = shutil.which('cargo')
    if not cargo:
        candidate = Path.home() / '.cargo' / 'bin' / ('cargo.exe' if os.name == 'nt' else 'cargo')
        if not candidate.exists():
            raise RuntimeError('Rust toolchain missing: install cargo before running checks')
        cargo = str(candidate)
    env = os.environ.copy()
    env['PATH'] = str(Path(cargo).parent) + os.pathsep + env.get('PATH', '')
    env['CARGO_TARGET_DIR'] = str(ROOT / 'target')
    env.pop('MOCK_VARIANT', None)
    def run(args, expected=0, variant=None):
        child_env = env.copy()
        if variant is not None:
            child_env['MOCK_VARIANT'] = variant
        print('+ ' + ' '.join(map(str,args)), flush=True)
        result = subprocess.run(args, cwd=ROOT, env=child_env, timeout=900)
        if result.returncode != expected:
            raise RuntimeError(f'check returned {result.returncode}, expected {expected}')
    for args in [
        ['fmt','--all','--','--check'],
        ['clippy','--locked','--all-targets','--','-D','warnings'],
        ['test','--locked','--all-targets'],
        ['build','--locked','--bins','--examples'],
    ]:
        run([cargo,*args])
    run([sys.executable,'tests/e2e.py'])
    executable = ROOT / 'target' / 'debug' / ('mcp-regress.exe' if os.name == 'nt' else 'mcp-regress')
    command = [str(executable),'check','--config','examples/fixture.toml','--baseline','fixtures/baseline.json']
    run(command)
    run(command, expected=2, variant='changed')
    print('All checks passed; reviewed baseline was not modified.',flush=True)

if __name__ == '__main__':
    try:
        main()
    except (OSError, RuntimeError, subprocess.TimeoutExpired) as error:
        print(f'Validation failed: {error}',file=sys.stderr)
        sys.exit(1)
