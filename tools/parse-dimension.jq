#!/usr/bin/jq --from-file
# cat DimensionDataBlock.json | ./parse-dimension.jq
[
    .Blocks[]
    | (if .name == "Dimension_Desert_dune_camp_01" then { r: 7, l: "C2"} else { r: 0, l: ""} end) as $level
    | .DimensionData.StaticTerminalPlacements[]?.LocalLogFiles[]?
    | select(.FileContent != 0 and .FileContent != "")
    | { rundown: $level.r, level: $level.l, name: .FileName, contentId: .FileContent, audioId: .AttachedAudioFile, zone: 0 }
]
| unique
| group_by(.contentId)
| map({ id: .[0].contentId, locations: group_by(.rundown, .level) | map({ rundown: .[0].rundown, level: .[0].level, zones: map(.zone), name: .[0].name }) })
