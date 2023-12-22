#!/usr/bin/jq --from-file
# cat LevelLayoutDataBlock.json | ./parse-level-layout.jq
[
    .Blocks[]
    | (.name | split("_") | first | sub("Rundown"; "") | sub("R"; "") | tonumber?) as $rundown
    | (.name | split("_") | nth(1) | sub("A2 BX Extension"; "AX") | sub("C4 DX Extension"; "CX")) as $level
    | .ZoneAliasStart as $zoneAliasStart
    | .Zones[]
    | select((.TerminalPlacements[]? | any) or (.SpecificTerminalSpawnDatas[]? | any))
    | (if .AliasOverride == -1 then
        .LocalIndex | sub("Zone_"; "") | tonumber + $zoneAliasStart
       else 
        .AliasOverride
       end) as $zone
    | ([ .TerminalPlacements[]?, .SpecificTerminalSpawnDatas[]? ]) | flatten | .[]
    | select(.LocalLogFiles[]? | any)
    | .LocalLogFiles[]
    | select(.FileContent != 0 and .FileContent != "")
    | { rundown: $rundown, level: $level, name: .FileName, contentId: .FileContent, audioId: .AttachedAudioFile, zone: $zone }
]
| unique
| group_by(.contentId)
| map({ id: .[0].contentId, locations: group_by(.rundown, .level) | map({ rundown: .[0].rundown, level: .[0].level, zones: map(.zone), name: .[0].name }) })
