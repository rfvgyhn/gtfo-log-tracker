#!/usr/bin/env sh

set -e

if [ ! -d "$1" ]; then
    echo "First param must be path to directory containing *DataBlock.json files"
    exit 1
fi

level_data_block="$1/LevelLayoutDataBlock.json"
if [ ! -f "$level_data_block" ]; then
    echo "Couldn't find '$level_data_block'"
    exit 1
fi

dimension_data_block="$1/DimensionDataBlock.json"
if [ ! -f "$dimension_data_block" ]; then
    echo "Couldn't find '$dimension_data_block'"
    exit 1
fi

level_transform="/tmp/level-layout-logs.json"
dimension_transform="/tmp/dimension-logs.json"

./parse-level-layout.jq < "$level_data_block" > "$level_transform"
./parse-dimension.jq < "$dimension_data_block" > "$dimension_transform"

jq -s 'add | sort_by(.locations.[0].rundown, .locations.[0].level, .locations.[0].zones)' "$level_transform" "$dimension_transform" > ../data/logs.json