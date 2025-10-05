#!/usr/bin/env bash
set -euo pipefail

cargo build

cd ruby
ruby main.rb
