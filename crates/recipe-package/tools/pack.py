"""Build a deterministic v1 package with the standard library and Rust validator.

Usage: python3 pack.py SOURCE OUTPUT --validator /path/to/validate-package
Requires Python 3.10+ on POSIX. SOURCE must be a dedicated package directory.
No output is published until the shared Rust validator accepts the snapshot.
"""
import argparse
import hashlib
import io
import os
from pathlib import Path
import re
import stat
import subprocess
import tempfile
import xml.etree.ElementTree as ET
import zipfile


def snapshot(source):
    files = {}
    total = 0
    visited = 0
    def walk(fd, prefix="", depth=0):
        nonlocal total, visited
        # Bound directory enumeration even when the source is not a package.
        with os.scandir(fd) as entries:
            for entry in entries:
                visited += 1
                if visited > 64:
                    raise ValueError("Too many directory entries")
                name = entry.name
                if (not re.fullmatch(r"[a-z0-9][a-z0-9_.-]{0,63}", name)
                        or name.split('.')[0] in {'con', 'prn', 'aux', 'nul'}
                        or re.fullmatch(r'(com|lpt)[1-9](\..*)?', name)):
                    raise ValueError("Noncanonical source path")
                path = prefix + name
                if len(path) > 128 or depth >= 4:
                    raise ValueError("Source path limit")
                info = entry.stat(follow_symlinks=False)
                if stat.S_ISDIR(info.st_mode):
                    child = os.open(name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
                    try: walk(child, path + '/', depth + 1)
                    finally: os.close(child)
                elif stat.S_ISREG(info.st_mode) and name.endswith('.xml'):
                    child = os.open(name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=fd)
                    with os.fdopen(child, 'rb') as stream:
                        if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
                            raise ValueError("Source changed type")
                        content = stream.read(32769)
                    total += len(content)
                    if len(content) > 32768 or total > 262144 or len(files) >= 16:
                        raise ValueError("Source size limit")
                    files[path] = content
                else:
                    raise ValueError("Only regular XML files and directories are allowed")
    fd = os.open(source, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try: walk(fd)
    finally: os.close(fd)
    return files


def build(source):
    files = snapshot(source)
    manifest = files.get('recipe.xml', b'').decode('utf-8-sig')
    # Accept only the leading XML 1.0 declaration, not arbitrary PIs.
    declaration = re.match(
        r"""^<\?xml[ \t\r\n]+version\s*=\s*(?P<v>['"])1\.0(?P=v)"""
        r"""(?:[ \t\r\n]+encoding\s*=\s*(?P<e>['"])(?i:UTF-8)(?P=e))?"""
        r"""(?:[ \t\r\n]+standalone\s*=\s*(?P<s>['"])(?:yes|no)(?P=s))?"""
        r"""[ \t\r\n]*\?>""", manifest)
    body = manifest[declaration.end():] if declaration else manifest
    if any(token in body for token in ('<!DOCTYPE', '<!ENTITY', '<?')):
        raise ValueError("Unsafe manifest XML")
    root = ET.fromstring(manifest)
    inventories = root.findall('assets')
    if len(inventories) != 1:
        raise ValueError("Exactly one asset inventory is required")
    expected = {'recipe.xml'}
    for asset in inventories[0]:
        path = asset.get('path')
        if asset.tag != 'asset' or path in expected or path not in files:
            raise ValueError("Missing, duplicate or invalid asset")
        expected.add(path)
        asset.set('bytes', str(len(files[path])))
        asset.set('sha256', hashlib.sha256(files[path]).hexdigest())
    if set(files) != expected:
        raise ValueError("Undeclared source files")
    # Stable serialization; executable XML bytes are never rewritten.
    files['recipe.xml'] = ET.tostring(root, encoding='utf-8', short_empty_elements=True)
    output = io.BytesIO()
    with zipfile.ZipFile(output, 'w', compression=zipfile.ZIP_STORED) as archive:
        for path, content in sorted(files.items()):
            info = zipfile.ZipInfo(path, (1980, 1, 1, 0, 0, 0))
            info.create_system = 3
            info.external_attr = 0o100644 << 16
            archive.writestr(info, content)
    data = output.getvalue()
    if len(data) > 524288:
        raise ValueError("Archive size limit")
    return data


def pack(source, output, validator):
    data = build(source)
    output = Path(output)
    # A sibling temporary file allows atomic, non-overwriting publication.
    fd, temporary = tempfile.mkstemp(prefix='.orcpkg-', dir=output.parent)
    try:
        with os.fdopen(fd, 'wb') as stream:
            stream.write(data)
        result = subprocess.run([str(Path(validator).resolve()), temporary],
                                capture_output=True, timeout=10, check=False)
        if result.returncode != 0:
            raise ValueError("Package rejected by the shared Rust validator")
        os.link(temporary, output)  # Fails if output already exists, including symlinks.
    finally:
        os.unlink(temporary)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source')
    parser.add_argument('output')
    parser.add_argument('--validator', required=True)
    args = parser.parse_args()
    try:
        pack(args.source, args.output, args.validator)
    except (OSError, ValueError, ET.ParseError, subprocess.TimeoutExpired) as error:
        parser.exit(1, f'Package not written: {error}\n')
