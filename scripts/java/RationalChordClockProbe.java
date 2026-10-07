// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.awt.Rectangle;
import com.fasterxml.jackson.databind.ObjectMapper;
public class RationalChordClockProbe extends ManualChordCacheProbe {
 static Object rows(List<Object> chords,List<Object[]> owners)throws Exception {
  List<Object> out=new ArrayList<>();int i=0;
  for(Object ch:chords){Object[] old=owners.get(i++);out.add(obj("head_bounds",rect(call(((List<?>)call(ch,"getMembers")).get(0),"getBounds")),"time_offset",RhythmResetProbe.rational(call(ch,"getTimeOffset")),"duration",RhythmResetProbe.rational(call(ch,"getDuration")),"end_time",RhythmResetProbe.rational(call(ch,"getEndTime")),"same_measure",call(ch,"getMeasure")==old[0],"same_part",call(ch,"getPart")==old[1],"same_voice",call(ch,"getVoice")==old[2],"same_slot",call(ch,"getSlot")==old[3]));}return out;
 }
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);Object[] a=load(args[0]);call(make("sheet.rhythm.PageRhythm",call(a[2],"getPage")),"reprocessStack",a[4]);List<Object> chords=new ArrayList<>((Collection<?>)call(a[5],"getHeadChords"));chords.sort(Comparator.comparingInt(ch->{try{return ((Rectangle)call(ch,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}));
  Object ch=chords.get(0),h=((List<?>)call(ch,"getMembers")).get(0),s1=call(chords.get(1),"getStem"),old=null;for(Object e:(Collection<?>)call(a[3],"edgesOf",h))if(e.getClass().getSimpleName().equals("HeadStemRelation")){old=e;break;}
  call(a[3],"addEdge",h,s1,ManualChordIdentityProbe.relation(old));
  Object otherMeasure=((List<?>)call(a[2],"getStacks")).get(1);Object other=call(otherMeasure,"getFirstMeasure");List<Object> more=new ArrayList<>((Collection<?>)call(other,"getHeadChords"));more.sort(Comparator.comparingInt(x->{try{return ((Rectangle)call(x,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}));chords.add(more.get(0));
  for(int[] pair:new int[][]{{0,2},{2,3},{3,4}})call(a[3],"addEdge",chords.get(pair[0]),chords.get(pair[1]),make("sig.relation.NextInVoiceRelation"));
  List<Object[]> owners=new ArrayList<>();for(Object x:chords)owners.add(new Object[]{call(x,"getMeasure"),call(x,"getPart"),call(x,"getVoice"),call(x,"getSlot")});Object expected=call(a[4],"getExpectedDuration");clean(a);List<Object> observations=new ArrayList<>();
  for(String time:new String[]{"0","1/64",null}){Object value=time==null?null:time.equals("0")?make("math.Rational",0,1):make("math.Rational",1,64);call(ch,"setAndPushTime",value);observations.add(obj("input",time,"chords",rows(chords,owners),"expected_duration",RhythmResetProbe.rational(call(a[4],"getExpectedDuration")),"same_expected",call(a[4],"getExpectedDuration")==expected,"book_dirty",call(a[0],"isDirty")));}
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","real retained manual HeadStem callback then declared NextInVoice chain, including cross-Measure endpoint; actual setAndPushTime/getDuration/getEndTime; no automatic recognition or rhythm reprocess/publication","observations",observations));System.exit(0);
 }
}
