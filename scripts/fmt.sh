#!/bin/bash

ROOTDIR=`cd $(dirname $0)/..; pwd`
cargo fmt "$@" -- --config=group_imports=StdExternalCrate --config=imports_granularity=Module