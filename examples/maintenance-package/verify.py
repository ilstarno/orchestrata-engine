"""Exact synthetic acceptance: python3 verify.py SOLVER PACKAGE [PACKER VALIDATOR]."""
import itertools
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import zipfile
from normalize import normalize

HERE = Path(__file__).resolve().parent

def exact(data, metric):
    jobs = [r['id'] for r in data['work_orders']]
    caps = {r['id']: min(3, r['capacity_slots']) for r in data['technicians'] if r['available']}
    costs = {(r['work_order_id'], r['technician_id']): r[metric] for r in data['eligibility'] if r['technician_id'] in caps}
    domains = [[None] + [w for j, w in costs if j == job] for job in jobs]
    best = None
    winners = []
    for plan in itertools.product(*domains):
        if any(plan.count(w) > cap for w, cap in caps.items()):
            continue
        score = (-sum(w is not None for w in plan), sum(costs[j, w] for j, w in zip(jobs, plan) if w is not None))
        if best is None or score < best:
            best, winners = score, [plan]
        elif score == best:
            winners.append(plan)
    return best, winners

def main(binary, archive, packer=None, validator=None):
    data = json.loads((HERE/'inputs.json').read_text())
    csv_records = normalize('csv', HERE/'sources/csv')
    connector_records = normalize('connector', HERE/'sources/connector-records.json')
    assert csv_records == connector_records == data
    with tempfile.TemporaryDirectory() as name:
        temp = Path(name)
        invalid = temp/'invalid.json'
        bad_source = {'tables': {k: {'records': v} for k, v in json.loads(json.dumps(data)).items()}}
        bad_source['tables']['technicians']['records'][0]['available'] = 'true'
        invalid.write_text(json.dumps(bad_source))
        try:
            normalize('connector', invalid)
            raise AssertionError('String boolean accepted')
        except ValueError:
            pass
        invalid.write_text('{"tables":{},"tables":{}}')
        try:
            normalize('connector', invalid)
            raise AssertionError('Duplicate key accepted')
        except ValueError:
            pass
        def run(records, mode, package=archive, reject=False):
            (temp/'inputs.json').write_text(json.dumps(records))
            p = subprocess.run([binary, str(package), str(temp/'inputs.json'), mode], capture_output=True, text=True, timeout=10)
            if reject:
                assert p.returncode != 0 and not p.stdout
                return
            assert p.returncode == 0, p.stderr
            return json.loads(p.stdout)
        def check(records, mode, metric):
            score, winners = exact(records, metric)
            output = run(records, mode)
            result = output['solution']
            assert (-result['assigned'], result['minimized_edge_sum']) == score
            assert [a['item_id'] for a in result['assignments']] == [j['id'] for j in records['work_orders']]
            assert tuple(a['resource_id'] for a in result['assignments']) in winners
            assert run(records, mode) == output
            return output, len(winners)
        baseline = {}
        for mode, metric in [('cost','cost_cents'), ('delay','delay_minutes')]:
            baseline[mode], ties = check(data, mode, metric)
            assert ties == 1
            assert run(csv_records, mode) == run(connector_records, mode) == baseline[mode]
            print(mode, json.dumps(baseline[mode]), 'exact_optima', ties)
        assert baseline['cost']['solution']['assignments'] != baseline['delay']['solution']['assignments']
        assert baseline['cost']['content_identity'] == baseline['delay']['content_identity']
        assert baseline['cost']['execution_identity'] != baseline['delay']['execution_identity']
        changed = json.loads(json.dumps(data)); changed['technicians'][0]['capacity_slots'] = 1
        output, _ = check(changed, 'cost', 'cost_cents')
        assert output['solution'] != baseline['cost']['solution']
        for t in changed['technicians']: t['available'] = False
        output, _ = check(changed,'cost','cost_cents'); assert output['solution']['assigned'] == 0
        bad = json.loads(json.dumps(data)); bad['technicians'][0]['capacity_slots'] = 'three'; run(bad,'cost',reject=True)
        bad = json.loads(json.dumps(data)); bad['eligibility'].append(dict(bad['eligibility'][0])); run(bad,'cost',reject=True)
        if packer and validator:
            source=temp/'source'; source.mkdir()
            with zipfile.ZipFile(archive) as z:
                for member in z.infolist():
                    path = Path(member.filename)
                    assert not path.is_absolute() and '..' not in path.parts
                    target=source/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(z.read(member))
            rebuilt=temp/'rebuilt.orcpkg'
            subprocess.run([sys.executable,packer,str(source),str(rebuilt),'--validator',validator],check=True)
            assert rebuilt.read_bytes() == Path(archive).read_bytes()
            model=source/'models/cost.xml';model.write_text(model.read_text().replace('field="cost_cents"/>','field="delay_minutes"/>'))
            altered=temp/'altered.orcpkg'
            subprocess.run([sys.executable,packer,str(source),str(altered),'--validator',validator],check=True)
            assert run(data,'cost',altered)['solution'] == baseline['delay']['solution']
        print('PASS: independent exact optima; changed inputs, availability and capacity; malformed/duplicate rejection; deterministic replay; optional package rebuild/objective edit.')

if __name__ == '__main__': main(*sys.argv[1:])
