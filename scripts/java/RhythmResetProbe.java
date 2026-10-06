// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class RhythmResetProbe extends FoundationOverlapProbe {
 static Object rational(Object value){return value==null?null:value.toString();}
 static Object state(Object measure)throws Exception {
  Object stack=call(measure,"getStack");List<Object> chords=new ArrayList<>();
  List<Object> all=new ArrayList<>();all.addAll((Collection<?>)call(measure,"getHeadChords"));all.addAll((Collection<?>)call(measure,"getRestChords"));
  all.sort(Comparator.comparingDouble(ch->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).getCenterX();}catch(Exception e){throw new RuntimeException(e);}}));
  for(Object ch:all){Object voice=call(ch,"getVoice"),slot=call(ch,"getSlot");chords.add(obj("bounds",rect(call(ch,"getBounds")),"duration",rational(call(ch,"getDuration")),"time_offset",rational(call(ch,"getTimeOffset")),"voice",voice==null?null:call(voice,"getId"),"slot_id",slot==null?null:call(slot,"getId")));}
  return obj("abnormal",call(measure,"isAbnormal"),"voice_count",((List<?>)call(measure,"getVoices")).size(),"stack_slot_count",((List<?>)call(stack,"getSlots")).size(),"expected",rational(call(stack,"getExpectedDuration")),"actual",rational(call(stack,"getActualDuration")),"chords",chords);
 }
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
  List<Object> measures=new ArrayList<>();for(Object sys:(List<?>)call(sheet,"getSystems"))for(Object stack:(List<?>)call(sys,"getStacks"))measures.add(call(stack,"getFirstMeasure"));
  Object measure=measures.get(Integer.parseInt(args[2])-1);call(measure,"setAbnormal",true);Object before=state(measure);
  call(measure,"resetRhythm");Object after=state(measure);call(measure,"resetRhythm");Object repeated=state(measure);
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","original saved-image measure with declared abnormal flag, actual resetRhythm/resetTiming plus repeat; not automatic full correction","before",before,"after",after,"repeated",repeated));System.exit(0);
 }
}
