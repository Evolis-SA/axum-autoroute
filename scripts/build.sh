#!/bin/bash

ROOTDIR=`cd $(dirname $0)/..; pwd`
RUSTFLAGS="-D warnings" cargo build "$@"