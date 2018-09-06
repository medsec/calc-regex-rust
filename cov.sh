#!/bin/bash

set -e

rm -rf target/cov

find target/debug -maxdepth 1 -type f -executable -exec kcov --include-pattern=code/src --exclude-pattern=tests target/cov {} \;
