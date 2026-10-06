// SPDX-License-Identifier: AGPL-3.0-or-later
// Existing original timed page; last-chord deletion and nullable end-time getter.
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class VoiceEndTimeProbe extends FoundationOverlapProbe {
 static Object state(Object voice)throws Exception {
  Object last=call(voice,"getLastChord"),end=last==null?null:call(last,"getEndTime");
  return obj("chord_count",((List<?>)call(voice,"getChords")).size(),"last_bounds",last==null?null:rect(call(last,"getBounds")),"last_duration",last==null?null:String.valueOf(call(last,"getDuration")),"last_offset",last==null?null:String.valueOf(call(last,"getTimeOffset")),"end_time",end==null?null:end.toString());
 }
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
  Object group=((Collection<?>)call(sig,"vertexSet")).stream().filter(n->n.getClass().getSimpleName().equals("BeamGroupInter")).findFirst().get(),voice=call(call(group,"getFirstChord"),"getVoice"),last=call(voice,"getLastChord");
  Object before=state(voice);call(last,"remove");Object after=state(voice);
  Object remaining=call(voice,"getLastChord");call(remaining,"setTimeOffset",(Object)null);Object missing=state(voice);
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","existing timed original Voice -> last AbstractChord.getEndTime before/after last chord removal and explicitly missing offset; not automatic underfilled measure export or complete Voice.checkDuration","before",before,"after_last_removal",after,"missing_offset",missing));System.exit(0);
 }
}
