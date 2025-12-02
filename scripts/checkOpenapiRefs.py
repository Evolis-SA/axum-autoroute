"""
Script to verify the consistency of references in openapi documents.
It will check that:
* Each schema which is referenced is present in the list of schemas.
* Each schema present in the list of schemas is referenced at least once.
"""

import os
import sys
import json

def check_refs(data):
    refs = find_refs(data)
    print(f"found {len(refs)} unique refs")

    n_error = 0
    for ref in refs:
        ok = check_ref_path(data, ref)
        if not ok:
            n_error += 1
    return (n_error, refs)


def find_refs(data):
    refs = set()
    if isinstance(data, dict):
        for key in data:
            if key == "$ref":
                refs.add(data[key])
            else:
                refs.update(find_refs(data[key]))
    elif isinstance(data, list):
        for i in range(len(data)):
            refs.update(find_refs(data[i]))
    return refs


def check_ref_path(data, ref):
    path = ref.split("/")
    if path[0] != "#":
        print(f"\tERROR: ref '{ref}' not starting with #/")
        return False

    cur = data
    for step in path[1:]:
        if step not in cur:
            print(f"\tERROR: '{step}' from ref '{ref}' does not exists")
            return False
        cur = cur[step]

    return True


def check_unused_schema(refs, data):
    schemas = set()
    if "components" in data and "schemas" in data["components"]:
        for schema_name in data["components"]["schemas"].keys():
            schemas.add(schema_name)

    refs_without_prefix = set(map(lambda s: s.removeprefix("#/components/schemas/"), refs))

    # remove schemas present in the refs, they are used
    schemas.difference_update(refs_without_prefix)
    
    for unused_schema in schemas:
        print(f"\tERROR: unused (not targeted by a reference) schema '{unused_schema}'")
    return len(schemas)


if __name__ == "__main__":
    if len(sys.argv) == 1 or "-h" in sys.argv or "--help" in sys.argv:
        print(f"USAGE: {os.path.basename(__file__)} [openapi_files.json ...]")
        print()
        print(f"DOCUMENTATION: {__doc__}")
        exit(0)

    n_error = 0
    for path in sys.argv[1:]:
        print(f"checking file '{path}'")
        with open(path, "r") as f:
            data = json.load(f)
            (n_unref, refs) = check_refs(data)
            n_unused = check_unused_schema(refs, data)

            n_error += n_unref + n_unused

    if n_error != 0:
        print(f"Check failed, {n_error} errors found")
        exit(1)
