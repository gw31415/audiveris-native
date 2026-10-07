// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;import com.fasterxml.jackson.databind.ObjectMapper;
public class VoiceRefineProbe extends StackReprocessProbe {
 static Object voices(Object measure)throws Exception {List<Object> rows=new ArrayList<>();for(Object v:(List<?>)call(measure,"getVoices")){Object first=call(v,"getFirstChord");rows.add(obj("id",call(v,"getId"),"first_chord_bounds",first==null?null:rect(call(first,"getBounds"))));}return rows;}
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);Object[] a=load(args[0]);Object page=call(a[2],"getPage");call(make("sheet.rhythm.PageRhythm",page),"reprocessStack",a[4]);Object measure=a[5],voice=((List<?>)call(measure,"getVoices")).get(0);call(voice,"setId",7);
  java.lang.reflect.Constructor<?> ctor=c("sheet.rhythm.Voice").getDeclaredConstructor();ctor.setAccessible(true);Object empty=ctor.newInstance();call(empty,"setId",99);call(measure,"addVoice",empty);
  Object before=voices(measure);c("sheet.rhythm.Voices").getMethod("refineSystem",c("sheet.SystemInfo")).invoke(null,a[2]);Object after=voices(measure);
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","actual PageRhythm.reprocessStack-created HIGH Voice, then declared ID7 plus actual Measure.addVoice empty Voice99, real Voices.refineSystem; not automatic image correction","before",before,"after",after,"rhythm",RhythmResetProbe.state(measure)));System.exit(0);
 }
}
