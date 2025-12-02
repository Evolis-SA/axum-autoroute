#!/bin/bash

ROOTDIR=`cd $(dirname $0)/..; pwd`
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --package axum-autoroute --all-features "$@"