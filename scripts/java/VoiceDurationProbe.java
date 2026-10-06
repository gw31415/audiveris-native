// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.lang.reflect.Field;
import com.fasterxml.jackson.databind.ObjectMapper;
public class VoiceDurationProbe extends FoundationOverlapProbe {
 static Object field(Object v,String name)throws Exception {Class<?> t=v.getClass();while(t!=null){try{Field f=t.getDeclaredField(name);f.setAccessible(true);return f.get(v);}catch(NoSuchFieldException e){t=t.getSuperclass();}}throw new NoSuchFieldException(name);}
 static String rational(Object v){return v==null?null:v.toString();}
 static Object state(Object voice)throws Exception {
  Object measure=call(voice,"getMeasure"),stack=call(measure,"getStack");
  return obj("termination",rational(call(voice,"getTermination")),"voice_excess",rational(field(voice,"excess")),"measure_abnormal",call(measure,"isAbnormal"),"stack_excess",rational(call(stack,"getExcess")),"expected_duration",rational(call(stack,"getExpectedDuration")));
 }
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
  Object group=((Collection<?>)call(sig,"vertexSet")).stream().filter(n->n.getClass().getSimpleName().equals("BeamGroupInter")).findFirst().get(),voice=call(call(group,"getFirstChord"),"getVoice");
  call(voice,"checkDuration");Object complete=state(voice);
  call(call(voice,"getLastChord"),"remove");call(voice,"checkDuration");Object shortened=state(voice);
  Object last=call(voice,"getLastChord");call(last,"setTimeOffset",c("math.Rational").getField("ONE").get(null));call(voice,"checkDuration");Object excess=state(voice);
  call(last,"setTimeOffset",(Object)null);call(voice,"checkDuration");Object missing=state(voice);
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","original normal timed Voice.checkDuration sequential stored-state controls: complete,last-chord removal,declared excess offset,missing offset; not automatic incomplete/excess measure correction or whole-measure-rest voice","complete",complete,"shortened",shortened,"excess",excess,"missing_offset",missing));System.exit(0);
 }
}
