// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.List;
import com.fasterxml.jackson.databind.ObjectMapper;

/** Actual source-first Page calls and deduplicated RhythmsStep impact. */
public class VoiceSelectedRhythmProbe extends VoiceResetRhythmProbe {
    public static void main(String[] args) throws Exception {
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),
            obj("scope","source-first actual Page stack processing and actual RhythmsStep.impact(Collection) with source,target,source insertion order and duplicate actual-stack suppression; ordinary owners, not full controller/history admission",
                "cases",observe(args[0],List.of("short_ordered","excess_ordered","short_impact","excess_impact"),true)));
    }
}
