// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.math.BigDecimal;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class CompoundVoiceExportProbe extends CompoundMappingProbe {
    static String duration(Object value, int wholeUnits) throws Exception {
        return make("math.Rational", ((BigDecimal) value).intValueExact(), wholeUnits).toString();
    }
    static Object inspectExport(Object book, Object measure, String name) throws Exception {
        Object mapping = inspectCompound(measure, name);
        StackSlotsProbe.clearTables(measure);
        StackSlotsProbe.generate(measure);
        invoke(make("sheet.rhythm.StackRhythm", call(measure, "getStack")), "readStackActualDuration");
        Object stackState = StackSlotsProbe.state(measure);
        Object score = ((List<?>) call(book, "getScores")).get(0);
        Object partwise = c("score.PartwiseBuilder").getMethod("build", c("score.Score")).invoke(null, score);
        Object part = ((List<?>) call(partwise, "getPart")).get(0);
        Object xmlMeasure = ((List<?>) call(part, "getMeasure")).get(1);
        int wholeUnits = (Integer) call(call(score, "getFirstPage"), "simpleDurationOf", make("math.Rational", 1, 1));
        List<Object> stream = new ArrayList<>();
        for (Object item : (List<?>) call(xmlMeasure, "getNoteOrBackupOrForward")) {
            String kind = item.getClass().getSimpleName();
            if (kind.equals("Note")) {
                Object pitch = call(item, "getPitch");
                if (pitch == null) throw new IllegalStateException("ordinary pitched saved control required");
                Object stem = call(item, "getStem");
                stream.add(obj("kind", "chord", "voice", Integer.parseInt((String) call(item, "getVoice")),
                        "step", call(pitch, "getStep").toString(), "octave", call(pitch, "getOctave"),
                        "type", call(call(item, "getType"), "getValue"), "stem", stem == null ? null : call(stem, "getValue").toString().toLowerCase(),
                        "duration", duration(call(item, "getDuration"), wholeUnits)));
            } else if (kind.equals("Backup")) {
                stream.add(obj("kind", "backup", "duration", duration(call(item, "getDuration"), wholeUnits)));
            } else if (kind.equals("Forward")) {
                stream.add(obj("kind", "forward", "voice", Integer.parseInt((String) call(item, "getVoice")),
                        "duration", duration(call(item, "getDuration"), wholeUnits)));
            }
        }
        Map result = obj("name", name, "stack", stackState, "export_stream", stream);
        if (name.equals("overlapping-continuation")) result.put("mapping", mapping);
        return result;
    }
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        Object book = c("sheet.Book").getMethod("loadBook", Path.class).invoke(null, Path.of(args[0]));
        Object sheet = call(((List<?>) call(book, "getStubs")).get(0), "getSheet");
        Object sys = ((List<?>) call(sheet, "getSystems")).get(0);
        Object measure = call(((List<?>) call(sys, "getStacks")).get(1), "getFirstMeasure");
        List<Object> controls = new ArrayList<>();
        for (String name : List.of("agreeing-times", "conflicting-times", "separate-member-times", "overlapping-continuation"))
            controls.add(inspectExport(book, measure, name));
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),
                obj("original_source", "9e1e55cd2746037d059345881c53e6a6754bffbd",
                        "scope", "same declared full compound mapping controls -> actual StackRhythm slots/Voice tables -> actual PartwiseBuilder.build score measure2 stream; not image polyphony admission",
                        "controls", controls));
        System.exit(0);
    }
}
