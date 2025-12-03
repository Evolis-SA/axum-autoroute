#!/bin/bash

set -e

ROOTDIR=`cd $(dirname $0)/..; pwd`

opt_features=("--no-default-features" "--all-features")

for opt_feature in ${opt_features[@]}; do
    echo "################################################################################"
    echo "## Check, $opt_feature"
    echo "################################################################################"
    RUSTFLAGS="-D warnings" cargo check $opt_feature
    echo

    echo "################################################################################"
    echo "## Clippy, $opt_feature"
    echo "################################################################################"
    ./scripts/clippy.sh $opt_feature
    echo

    echo "################################################################################"
    echo "## Tests, $opt_feature"
    echo "################################################################################"
    ./scripts/test.sh $opt_feature
    echo
done

echo "################################################################################"
echo "## Check OpenApiRefs"
echo "################################################################################"
python3 "$ROOTDIR/scripts/checkOpenapiRefs.py" $ROOTDIR/example/refs/openapi/*.json

