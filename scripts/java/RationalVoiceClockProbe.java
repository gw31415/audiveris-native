// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.awt.Rectangle;
import com.fasterxml.jackson.databind.ObjectMapper;
public class RationalVoiceClockProbe extends ManualChordCacheProbe {
 static Object headBounds(Object ch)throws Exception {return rect(call(((List<?>)call(ch,"getMembers")).get(0),"getBounds"));}
 static Object timing(Object[] a)throws Exception {
  List<?> slots=(List<?>)call(a[4],"getSlots");List<Object> slotRows=new ArrayList<>(),voices=new ArrayList<>();
  for(Object s:slots){List<Object> heads=new ArrayList<>();for(Object ch:(Collection<?>)call(s,"getChords"))heads.add(headBounds(ch));slotRows.add(obj("id",call(s,"getId"),"time_offset",RhythmResetProbe.rational(call(s,"getTimeOffset")),"x_offset",call(s,"getXOffset"),"incoming_heads",heads));}
  for(Object v:(List<?>)call(a[5],"getVoices")){List<Object> heads=new ArrayList<>(),table=new ArrayList<>();for(Object ch:(Collection<?>)call(v,"getChords"))heads.add(headBounds(ch));for(Object s:slots){Object info=call(v,"getSlotInfo",s);if(info!=null)table.add(obj("slot_id",call(s,"getId"),"status",info.getClass().getField("status").get(info).toString(),"head_bounds",headBounds(info.getClass().getField("chord").get(info))));}voices.add(obj("id",call(v,"getId"),"heads",heads,"duration",RhythmResetProbe.rational(call(v,"getDuration")),"end_time",RhythmResetProbe.rational(call(call(v,"getLastChord"),"getEndTime")),"termination",RhythmResetProbe.rational(call(v,"getTermination")),"excess",RhythmResetProbe.rational(flagIn(v,"sheet.rhythm.Voice","excess")),"slots",table));}
  return obj("expected",RhythmResetProbe.rational(call(a[4],"getExpectedDuration")),"actual",RhythmResetProbe.rational(call(a[4],"getActualDuration")),"stack_excess",RhythmResetProbe.rational(call(a[4],"getExcess")),"abnormal",call(a[5],"isAbnormal"),"voices",voices,"slots",slotRows);
 }
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);Object[] a=load(args[0]);call(make("sheet.rhythm.PageRhythm",call(a[2],"getPage")),"reprocessStack",a[4]);List<Object> chords=new ArrayList<>((Collection<?>)call(a[5],"getHeadChords"));chords.sort(Comparator.comparingInt(ch->{try{return ((Rectangle)call(ch,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}));Object ch=chords.get(0),h=((List<?>)call(ch,"getMembers")).get(0),s1=call(chords.get(1),"getStem"),old=null;for(Object e:(Collection<?>)call(a[3],"edgesOf",h))if(e.getClass().getSimpleName().equals("HeadStemRelation")){old=e;break;}
  Object expected=call(a[4],"getExpectedDuration");call(a[3],"addEdge",h,s1,ManualChordIdentityProbe.relation(old));call(make("sheet.rhythm.PageRhythm",call(a[2],"getPage")),"reprocessStack",a[4]);Object after=timing(a),stream=music(a[0]);
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","real saved owner manual HeadStem callback followed by actual PageRhythm.reprocessStack, Voice/Slot getters and PartwiseBuilder; native comparison may declare mapped Voice/chord schedules, not claim automatic image/reprocess success","quarter_divisions",call(call(a[2],"getPage"),"simpleDurationOf",make("math.Rational",1,4)),"same_expected",call(a[4],"getExpectedDuration")==expected,"timing",after,"original_export_stream",stream));System.exit(0);
 }
}
