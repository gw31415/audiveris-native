// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.lang.reflect.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class SlotDurationProbe extends StackSlotsProbe {
 static Object duration(Object measure)throws Exception {
  Object stack=call(measure,"getStack"),rhythm=make("sheet.rhythm.StackRhythm",stack);
  Method read=rhythm.getClass().getDeclaredMethod("readStackActualDuration");read.setAccessible(true);read.invoke(rhythm);
  List<Object> voices=new ArrayList<>();
  for(Object voice:(List<?>)call(measure,"getVoices")) voices.add(obj("id",call(voice,"getId"),"duration",String.valueOf(call(voice,"getDuration"))));
  return obj("voices",voices,"slots_duration",String.valueOf(call(stack,"getSlotsDuration")),"actual_duration",String.valueOf(call(stack,"getActualDuration")));
 }
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
  Object group=((Collection<?>)call(sig,"vertexSet")).stream().filter(n->n.getClass().getSimpleName().equals("BeamGroupInter")).findFirst().get(),voice=call(call(group,"getFirstChord"),"getVoice"),measure=call(voice,"getMeasure"),stack=call(measure,"getStack");
  clearTables(measure);generate(measure);Object saved=duration(measure);
  Object first=call(voice,"getFirstChord");call(first,"setTimeOffset",c("math.Rational").getField("ONE").get(null));clearTables(measure);generate(measure);Object max=duration(measure);
  call(first,"setTimeOffset",c("math.Rational").getField("ZERO").get(null));clearTables(measure);generate(measure);
  call(((List<?>)call(stack,"getSlots")).get(0),"setTimeOffset",(Object)null);Object missing=duration(measure);
  ((List<?>)call(stack,"getSlots")).clear();Object noSlots=duration(measure);
  call(measure,"resetRhythm");Object empty=duration(measure);
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","saved normal Voice/stack slot durations, declared non-last chord maximum, nullable slot time, no slots with voice (expected fallback), no voices (zero); not automatic whole-rest Voice or polyphony", "saved",saved,"maximum",max,"missing_slot_time",missing,"no_slots_with_voice",noSlots,"no_voices",empty));System.exit(0);
 }
}
