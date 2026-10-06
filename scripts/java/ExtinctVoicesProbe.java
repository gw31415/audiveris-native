// SPDX-License-Identifier: AGPL-3.0-or-later
// Original saved-image progression and declared Voice lifetime controls.
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class ExtinctVoicesProbe extends RookieMappingProbe {
 static Object controls(Object measure)throws Exception {
  Object sys=call(call(measure,"getStack"),"getSystem"),sig=call(sys,"getSig");
  List<Object> cases=new ArrayList<>();
  for(String name:List.of("gap-expired","equal-end-boundary","next-chain-to-later","next-only-before-current","already-extinct-direct-rookie","unknown-last-clock","unknown-first-clock")) {
   call(measure,"resetRhythm");List<Object> all=new ArrayList<>((Collection<?>)call(measure,"getStandardChords"));
   Object rhythm=make("sheet.rhythm.MeasureRhythm",measure),retriever=make("sheet.rhythm.SlotsRetriever",measure,false);put(rhythm,"narrowSlotsRetriever",retriever);
   List<?> narrow=(List<?>)call(retriever,"buildSlots"),wide=(List<?>)call(make("sheet.rhythm.SlotsRetriever",measure,true),"buildSlots"),slots=(List<?>)invoke(rhythm,"buildCompoundSlots",narrow,wide);((List)field(rhythm,"slots")).addAll(slots);invoke(rhythm,"processStartingChords");
   Object current=slots.get(2),first=((List<?>)call(current,"getMembers")).get(0),voice=call(all.get(0),"getVoice");
   Object time=name.equals("unknown-first-clock")?null:make("math.Rational",1,name.equals("equal-end-boundary")?4:2);call(first,"setTimeOffset",time);
   if(name.equals("unknown-last-clock"))call(all.get(0),"setTimeOffset",(Object)null);
   List<Object> added=new ArrayList<>();
   if(name.equals("next-chain-to-later")||name.equals("next-only-before-current")){Object rel=make("sig.relation.NextInVoiceRelation");call(sig,"addEdge",all.get(0),all.get(1),rel);added.add(rel);}
   if(name.equals("next-chain-to-later")){Object rel=make("sig.relation.NextInVoiceRelation");call(sig,"addEdge",all.get(1),all.get(3),rel);added.add(rel);}
   if(name.equals("already-extinct-direct-rookie")){((Set)field(rhythm,"extinctVoices")).add(voice);Object rel=make("sig.relation.NextInVoiceRelation");call(sig,"addEdge",all.get(0),all.get(2),rel);added.add(rel);}
   List<Integer> before=new ArrayList<>();for(Object v:(Collection<?>)field(rhythm,"extinctVoices"))before.add((Integer)call(v,"getId"));Collections.sort(before);
   List<Object> facts=new ArrayList<>();for(Object ch:all){Object next=call(ch,"getNextChordInVoiceSequence"),slot=call(ch,"getSlot");facts.add(obj("id",all.indexOf(ch),"same_measure",call(ch,"getMeasure")==measure,"time_offset",fraction(call(ch,"getTimeOffset")),"duration",fraction(call(ch,"getDuration")),"slot_x",((Number)call(slot,"getXOffset")).longValue(),"next",next==null?null:all.indexOf(next)));}
   invoke(rhythm,"purgeExtinctVoices",current);List<Integer> after=new ArrayList<>();for(Object v:(Collection<?>)field(rhythm,"extinctVoices"))after.add((Integer)call(v,"getId"));Collections.sort(after);
   Class<?> mapperClass=c("sheet.rhythm.MeasureRhythm$SlotMapper");var ctor=mapperClass.getDeclaredConstructor(c("sheet.rhythm.MeasureRhythm"),c("sheet.rhythm.Slot$CompoundSlot"));ctor.setAccessible(true);Object mapper=ctor.newInstance(rhythm,current);invoke(mapper,"purgeRookiesSet");List<?> rookies=(List<?>)field(mapper,"rookies"),actives=(List<?>)invoke(mapper,"retrieveActives");List<Object> nexts=new ArrayList<>();for(Object p:(Collection<?>)field(mapper,"nextList"))nexts.add(obj("rookie",all.indexOf(field(p,"rookie")),"active",all.indexOf(field(p,"active"))));
   cases.add(obj("name",name,"first_time",fraction(time),"current_x",((Number)call(current,"getXOffset")).longValue(),"chords",facts,"voices",List.of(obj("id",call(voice,"getId"),"measure_rest",false,"last_chord",all.indexOf(call(voice,"getLastChord")))),"extinct_before",before,"extinct_after",after,"measure_abnormal",call(measure,"isAbnormal"),"rookies",ids(rookies,all),"next",nexts,"actives",ids(actives,all)));
   for(Object rel:added)call(sig,"removeEdge",rel);
  }
  return cases;
 }
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);List<Object> pages=new ArrayList<>();Object control=null;
  for(int i=1;i<args.length;i++){Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[i])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");List<Object> measures=new ArrayList<>();for(Object sys:(List<?>)call(sheet,"getSystems"))for(Object stack:(List<?>)call(sys,"getStacks")){Object measure=call(stack,"getFirstMeasure");Map observed=(Map)inspect(measure,true);measures.add(obj("number",observed.get("number"),"extinction_steps",observed.get("extinction_steps")));if(i==1&&control==null&&((Integer)call(stack,"getIdValue"))==2)control=measure;}pages.add(obj("input",Path.of(args[i]).getFileName().toString(),"measures",measures));}
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[0]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","two saved successful same-image original progressions; declared seven lifetime/NextInVoice controls on actual chord objects are not native image polyphony","pages",pages,"controls",controls(control)));System.exit(0);
 }
}
