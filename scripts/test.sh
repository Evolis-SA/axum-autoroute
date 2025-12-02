#!/bin/bash

ROOTDIR=`cd $(dirname $0)/..; pwd`
cargo test "$@"