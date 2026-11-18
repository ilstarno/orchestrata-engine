"""Local source-neutral adapter example; no network or credentials.

normalize.py connector connector-records.json
normalize.py csv sources/csv
The mathematical runtime consumes the same typed named records from either.
"""
import csv
import json
from pathlib import Path
import sys

SCHEMA = {
    'work_orders': {'id': str},
    'technicians': {'id': str, 'capacity_slots': int, 'available': bool},
    'eligibility': {'work_order_id': str, 'technician_id': str, 'cost_cents': int, 'delay_minutes': int},
}

def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError('Duplicate JSON key')
        result[key] = value
    return result


def normalize(channel, source):
    source = Path(source)
    if channel == 'connector':
        if source.stat().st_size > 524288:
            raise ValueError('Input byte limit')
        payload = json.loads(source.read_text(), object_pairs_hook=unique_object)
        if set(payload) != {'tables'} or set(payload['tables']) != set(SCHEMA):
            raise ValueError('Unexpected named tables')
        tables = {}
        for name, table in payload['tables'].items():
            if set(table) != {'records'} or not isinstance(table['records'], list):
                raise ValueError('Expected records array')
            tables[name] = table['records']
    elif channel == 'csv':
        tables = {}
        for name, fields in SCHEMA.items():
            path = source/(name+'.csv')
            if path.stat().st_size > 524288:
                raise ValueError('Input byte limit')
            with path.open(newline='') as stream:
                reader = csv.DictReader(stream)
                if reader.fieldnames != list(fields):
                    raise ValueError('Unexpected, duplicate or reordered CSV headers')
                tables[name] = list(reader)
    else:
        raise ValueError('Unknown source channel')
    if sum(len(rows) for rows in tables.values()) > 128:
        raise ValueError('Record limit')
    result = {}
    for name, rows in tables.items():
        result[name] = []
        for row in rows:
            if not isinstance(row, dict) or set(row) != set(SCHEMA[name]):
                raise ValueError('Unexpected fields')
            typed = {}
            for field, kind in SCHEMA[name].items():
                value = row[field]
                if channel == 'csv' and kind is bool:
                    if value not in ('true','false'): raise ValueError('Expected true or false')
                    value = value == 'true'
                elif channel == 'csv' and kind is int:
                    if not isinstance(value,str) or not value.isascii() or not value.isdecimal():
                        raise ValueError('Expected unsigned decimal integer')
                    value = int(value)
                if type(value) is not kind or kind is int and not 0 <= value <= 4294967295:
                    raise ValueError('Invalid typed field')
                typed[field] = value
            result[name].append(typed)
    return result

if __name__ == '__main__':
    print(json.dumps(normalize(*sys.argv[1:]), indent=2))
