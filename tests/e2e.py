"""Real-process CLI contract tests. Run after cargo build --bins --examples."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time
import signal
import unittest

ROOT = Path(__file__).resolve().parents[1]
EXT = '.exe' if os.name == 'nt' else ''
CLI = ROOT / 'target' / 'debug' / ('mcp-regress' + EXT)
SERVER = ROOT / 'target' / 'debug' / 'examples' / ('mock_server' + EXT)

class CliTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.config = self.root / 'fixture.toml'
        self.baseline = self.root / 'baseline.json'
        self.configure()

    def configure(self, variant='', extra='', case_extra=''):
        self.config.write_text('version=1\n[server]\ncommand=' + json.dumps(str(SERVER)) +
            '\ntimeout_ms=500\nrun_timeout_ms=2000\n' + extra + '\n[server.env]\nMOCK_VARIANT=' + json.dumps(variant) +
            '\n[[cases]]\nid="echo"\ntool="echo"\narguments={text="hello"}\n' + case_extra)

    def run_cli(self, *args, expected=0):
        process = subprocess.run([str(CLI), *map(str,args)], cwd=ROOT, capture_output=True, text=True, timeout=10)
        self.assertEqual(process.returncode, expected, process.stdout + process.stderr)
        self.assertNotIn('CANARY_SECRET', process.stdout + process.stderr)
        return process

    def record(self, expected=0):
        return self.run_cli('baseline','--config',self.config,'--output',self.baseline,expected=expected)

    def test_validate_never_launches_server(self):
        self.configure('hang')
        self.run_cli('validate','--config',self.config)

    def test_report_cannot_overwrite_baseline(self):
        self.record(); original=self.baseline.read_bytes()
        self.run_cli('check','--config',self.config,'--baseline',self.baseline,'--json',self.baseline,expected=1)
        self.assertEqual(original,self.baseline.read_bytes())

    def test_duplicate_reports_fail_before_run(self):
        self.record(); report=self.root/'report'
        self.run_cli('check','--config',self.config,'--baseline',self.baseline,'--json',report,'--markdown',report,expected=1)
        self.assertFalse(report.exists())

    def test_policy_overrides_and_expired_exceptions(self):
        self.record(); self.configure('changed')
        policy=self.root/'policy.toml'
        policy.write_text('version=1\n[overrides]\n"case.changed"="warning"')
        self.run_cli('check','--config',self.config,'--baseline',self.baseline,'--policy',policy)
        self.run_cli('check','--config',self.config,'--baseline',self.baseline,'--policy',policy,'--fail-on','any',expected=2)
        policy.write_text('version=1\n[[exceptions]]\nrule="case.changed"\nname="echo"\nreason="approved"\nexpires="2000-01-01"')
        self.run_cli('check','--config',self.config,'--baseline',self.baseline,'--policy',policy,'--fail-on','none',expected=1)

    def test_exception_is_retained_in_json_report(self):
        self.record(); self.configure('changed')
        policy=self.root/'policy.toml'; report=self.root/'report.json'
        policy.write_text('version=1\n[[exceptions]]\nrule="case.changed"\nname="echo"\nreason="approved migration"\nexpires="2999-01-01"')
        self.run_cli('check','--config',self.config,'--baseline',self.baseline,'--policy',policy,'--json',report)
        changes=json.loads(report.read_text())['changes']
        self.assertTrue(any(c.get('exception',{}).get('reason')=='approved migration' for c in changes))

    def test_clean_and_changed_gate(self):
        self.record()
        self.run_cli('check','--config',self.config,'--baseline',self.baseline)
        self.configure('changed')
        self.run_cli('check','--config',self.config,'--baseline',self.baseline, expected=2)

    def test_deterministic_capture(self):
        self.record(); before=self.baseline.read_bytes(); self.record()
        self.assertEqual(before,self.baseline.read_bytes())

    def test_rpc_failure_never_becomes_baseline(self):
        self.configure('rpc_error'); self.record(expected=1)
        self.assertFalse(self.baseline.exists())

    def test_expected_rpc_error_is_explicit(self):
        self.configure('rpc_error',case_extra='expected_rpc_error=-32602')
        self.record(); self.assertNotIn('CANARY_SECRET',self.baseline.read_text())
        self.run_cli('check','--config',self.config,'--baseline',self.baseline)

    def test_missing_expected_rpc_error_fails(self):
        self.configure(case_extra='expected_rpc_error=-32602'); self.record(expected=1)

    def test_wrong_response_id_fails(self):
        self.configure('wrong_id'); self.record(expected=1)

    def test_oversized_frame_fails(self):
        self.configure('oversized',extra='max_frame_bytes=1024'); self.record(expected=1)

    def test_timeout_does_not_replace_existing_baseline(self):
        self.record(); before=self.baseline.read_bytes()
        self.configure('hang'); self.record(expected=1)
        self.assertEqual(before,self.baseline.read_bytes())

    def test_total_budget(self):
        self.configure('hang'); text=self.config.read_text().replace('timeout_ms=500','timeout_ms=5000').replace('run_timeout_ms=2000','run_timeout_ms=100')
        self.config.write_text(text)
        self.assertIn('total run deadline', self.record(expected=1).stderr)

    @unittest.skipUnless(os.name == 'posix' and Path('/proc').exists(), 'Linux process state evidence')
    def test_timeout_terminates_process_group(self):
        pid_file=self.root / 'pid'
        self.configure('tree_hang')
        text=self.config.read_text().replace('[server.env]', '[server.env]\nMOCK_PID_FILE='+json.dumps(str(pid_file)))
        self.config.write_text(text)
        self.record(expected=1)
        pid=int(pid_file.read_text())
        # Orphaned zombies can await init reaping; they are no longer running.
        for _ in range(20):
            stat=Path('/proc') / str(pid) / 'stat'
            if not stat.exists() or stat.read_text().split(') ')[1].startswith('Z'):
                return
            time.sleep(0.05)
        self.fail('descendant survived timeout')

    @unittest.skipUnless(os.name == 'posix' and Path('/proc').exists(), 'Linux SIGINT process evidence')
    def test_sigint_cancels_and_terminates_descendants(self):
        pid_file=self.root/'pid'
        self.configure('tree_hang')
        text=self.config.read_text().replace('timeout_ms=500','timeout_ms=10000').replace('run_timeout_ms=2000','run_timeout_ms=20000')
        text=text.replace('[server.env]', '[server.env]\nMOCK_PID_FILE='+json.dumps(str(pid_file)))
        self.config.write_text(text)
        process=subprocess.Popen([str(CLI),'baseline','--config',str(self.config),'--output',str(self.baseline)],cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        try:
            deadline=time.monotonic()+5
            while not pid_file.exists() and time.monotonic()<deadline:
                time.sleep(0.02)
            self.assertTrue(pid_file.exists(),'test server never started')
            process.send_signal(signal.SIGINT)
            stdout,stderr=process.communicate(timeout=5)
            self.assertEqual(process.returncode,1,stdout+stderr)
            self.assertIn('cancelled',stderr)
            self.assertFalse(self.baseline.exists())
            stat=Path('/proc')/pid_file.read_text()/'stat'
            for _ in range(50):
                if not stat.exists() or stat.read_text().split(') ')[1].startswith('Z'):
                    return
                time.sleep(0.02)
            self.fail('descendant survived SIGINT')
        finally:
            if process.poll() is None:
                process.kill(); process.communicate(timeout=5)

    def test_tampered_snapshot_fails_even_with_fail_on_none(self):
        self.record(); doc=json.loads(self.baseline.read_text()); doc['tools'][0]['name']='tampered'
        self.baseline.write_text(json.dumps(doc))
        self.run_cli('diff','--baseline',self.baseline,'--current',self.baseline,'--fail-on','none',expected=1)

    def test_missing_ignore_pointer_fails(self):
        self.configure(case_extra='ignore_pointers=["/does-not-exist"]'); self.record(expected=1)

    def test_redaction_before_persistence(self):
        text=self.config.read_text().replace('text="hello"','text="CANARY_SECRET",token="CANARY_SECRET"')
        # The echo tool returns text. An explicit pointer prevents persisting it.
        self.config.write_text(text+'ignore_pointers=["/content/0/text"]\n'); self.record()
        self.assertNotIn('CANARY_SECRET',self.baseline.read_text())

if __name__ == '__main__':
    unittest.main()
