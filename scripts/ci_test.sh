#!/bin/bash

set -e

ROOTDIR=`cd $(dirname $0)/..; pwd`

opt_features=("--no-default-features" "--all-features")
crate_dirs=("example" "lib" "macros")

for opt_feature in ${opt_features[@]}; do
    for crate in ${crate_dirs[@]}; do
        opt_manifest="--manifest-path $ROOTDIR/$crate/Cargo.toml"

        echo "################################################################################"
        echo "## Check, $opt_feature, $opt_manifest"
        echo "################################################################################"
        RUSTFLAGS="-D warnings" cargo check $opt_feature $opt_manifest
        echo

        echo "################################################################################"
        echo "## Clippy, $opt_feature, $opt_manifest"
        echo "################################################################################"
        ./scripts/clippy.sh $opt_feature $opt_manifest
        echo
    done
    

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

echo "################################################################################"
echo "## Doc"
echo "################################################################################"
./scripts/doc.sh

echo "################################################################################"
echo "## Format"
echo "################################################################################"
./scripts/fmt.sh

echo "################################################################################"
echo "## Cargo deny"
echo "################################################################################"
cargo deny check
