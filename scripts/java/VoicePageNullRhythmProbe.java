// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.List;
import com.fasterxml.jackson.databind.ObjectMapper;

/** Rebuild the next real stack with a genuinely reset unselected source. */
public class VoicePageNullRhythmProbe extends VoiceResetRhythmProbe {
    public static void main(String[] args) throws Exception {
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),
            obj("scope","actual PageRhythm next-stack rebuild with null previous Measure source after genuine reset; observed original exception effects, not atomic rollback parity",
                "cases",observe(args[0],List.of("excess_page_null"),true)));
    }
}
