// SPDX-License-Identifier: AGPL-3.0-or-later
// Direct original compound partition/getter observations, not full Voice matching.
import java.nio.file.Path;import java.util.*;import java.lang.reflect.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class CompoundSlotsProbe extends FoundationOverlapProbe {
 static Object inspect(Object measure,boolean controls)throws Exception {
  call(measure,"resetRhythm");List<Object> chords=new ArrayList<>((Collection<?>)call(measure,"getStandardChords"));
  List<?> narrow=(List<?>)call(make("sheet.rhythm.SlotsRetriever",measure,false),"buildSlots"),wide=(List<?>)call(make("sheet.rhythm.SlotsRetriever",measure,true),"buildSlots");
  List<Object> cases=new ArrayList<>();cases.add(caseRow("actual-retrieved",measure,chords,narrow,wide));
  if(controls){List<Object> singles=new ArrayList<>();for(int i=0;i<3;i++)singles.add(make("sheet.rhythm.Slot$MeasureSlot",i+1,measure,List.of(chords.get(i))));
   List<Object> merged=List.of(make("sheet.rhythm.Slot$MeasureSlot",1,measure,chords.subList(0,2)),make("sheet.rhythm.Slot$MeasureSlot",2,measure,List.of(chords.get(2))));
   cases.add(caseRow("declared-two-narrow-members",measure,chords,singles,merged));
   List<Object> overlapping=List.of(make("sheet.rhythm.Slot$MeasureSlot",1,measure,chords.subList(0,3)),make("sheet.rhythm.Slot$MeasureSlot",2,measure,List.of(chords.get(0))));
   cases.add(caseRow("declared-overlapping-wide-tail-original-cursor",measure,chords,singles,overlapping));
   List<Object> ordinate=List.of(make("sheet.rhythm.Slot$MeasureSlot",1,measure,List.of(chords.get(4),chords.get(0))),make("sheet.rhythm.Slot$MeasureSlot",2,measure,List.of(chords.get(5),chords.get(1))));
   Object wideOrder=make("sheet.rhythm.Slot$MeasureSlot",1,measure,List.of(chords.get(4),chords.get(1),chords.get(5),chords.get(0)));
   cases.add(caseRow("declared-ordinate-order-and-lookup",measure,chords,ordinate,List.of(wideOrder)));
   cases.add(caseRow("declared-empty",measure,chords,List.of(),List.of()));
  }
  return obj("number",call(call(measure,"getStack"),"getIdValue"),"chord_bounds",chords.stream().map(ch->{try{return rect(call(ch,"getBounds"));}catch(Exception e){throw new RuntimeException(e);}}).toList(),"cases",cases);
 }
 static List<Object> slots(List<?> slots,List<Object> chords)throws Exception {List<Object> out=new ArrayList<>();for(Object slot:slots){List<Object> ids=new ArrayList<>();for(Object ch:(List<?>)call(slot,"getChords"))ids.add(chords.indexOf(ch));out.add(obj("id",call(slot,"getId"),"chord_ids",ids));}return out;}
 static Object caseRow(String name,Object measure,List<Object> chords,List<?> narrow,List<?> wide)throws Exception {
  Object rhythm=make("sheet.rhythm.MeasureRhythm",measure);Method method=c("sheet.rhythm.MeasureRhythm").getDeclaredMethod("buildCompoundSlots",List.class,List.class);method.setAccessible(true);List<?> compounds=(List<?>)method.invoke(rhythm,narrow,wide);List<Object> result=new ArrayList<>();
  for(Object compound:compounds){List<Object> ids=new ArrayList<>(),members=new ArrayList<>(),lookup=new ArrayList<>();for(Object ch:(List<?>)call(compound,"getChords")){ids.add(chords.indexOf(ch));Object slot=call(compound,"getNarrowSlot",ch);lookup.add(obj("chord_id",chords.indexOf(ch),"narrow_slot_id",slot==null?null:call(slot,"getId")));}for(Object s:(List<?>)call(compound,"getMembers"))members.add(call(s,"getId"));result.add(obj("id",call(compound,"getId"),"narrow_slot_ids",members,"chord_ids",ids,"lookup",lookup));}
  return obj("name",name,"narrow",slots(narrow,chords),"wide",slots(wide,chords),"compounds",result);
 }
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);List<Object> pages=new ArrayList<>();
  for(int i=1;i<args.length;i++){Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[i])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");List<Object> measures=new ArrayList<>();boolean first=true;for(Object sys:(List<?>)call(sheet,"getSystems"))for(Object stack:(List<?>)call(sys,"getStacks")){measures.add(inspect(call(stack,"getFirstMeasure"),i==1&&first));first=false;}pages.add(obj("input",Path.of(args[i]).getFileName().toString(),"measures",measures));}
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[0]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","original saved-state MeasureRhythm.buildCompoundSlots/Slot.chordsOf/getNarrowSlot; four declared controls separate from actual image recognition/general Voice matching","pages",pages));System.exit(0);
 }
}
