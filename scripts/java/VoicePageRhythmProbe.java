// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.List;
import com.fasterxml.jackson.databind.ObjectMapper;

/** Actual PageRhythm stack rebuild then system refinement after genuine reset. */
public class VoicePageRhythmProbe extends VoiceResetRhythmProbe {
    public static void main(String[] args) throws Exception {
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),
            obj("scope","actual PageRhythm.reprocessStack after genuine Measure reset, cached expectedDuration, actual rebuilt Voice/Slot references and system tie/beam/Next/preferred refinement; ordinary controls, not general history/controller admission",
                "cases",observe(args[0],List.of("short_page","excess_page"),true)));
    }
}
