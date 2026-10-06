// SPDX-License-Identifier: AGPL-3.0-or-later
// Original live SlotMapper progression on already saved successful image states.
import java.nio.file.Path;import java.util.*;import java.lang.reflect.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class RookieMappingProbe extends FoundationOverlapProbe {
 static Object invoke(Object o,String name,Object...args)throws Exception {for(Method m:o.getClass().getDeclaredMethods())if(m.getName().equals(name)&&m.getParameterCount()==args.length){m.setAccessible(true);return m.invoke(o,args);}throw new NoSuchMethodException(name);}
 static Object field(Object o,String name)throws Exception {Field f=o.getClass().getDeclaredField(name);f.setAccessible(true);return f.get(o);}
 static void put(Object o,String name,Object v)throws Exception {Field f=o.getClass().getDeclaredField(name);f.setAccessible(true);f.set(o,v);}
 static List<Object> ids(Collection<?> chords,List<Object> all){return chords.stream().map(ch->(Object)all.indexOf(ch)).toList();}
 static Object fraction(Object v){return v==null?null:v.toString();}
 static Object distanceFact(Object ch)throws Exception {
  boolean head=ch.getClass().getSimpleName().equals("HeadChordInter");Object voice=call(ch,"getVoice");Object heads=head?call(ch,"getHeadsBounds"):call(ch,"getBounds");java.awt.Rectangle box=(java.awt.Rectangle)heads;
  return obj("head",head,"staff",call(call(ch,"getTopStaff"),"getId"),"part",call(call(ch,"getPart"),"getId"),"stem_dir",call(ch,"getStemDir"),"voice",voice==null?null:call(voice,"getId"),"starting_staff",voice==null?null:call(call(voice,"getStartingStaff"),"getId"),"absolute_pitch",head?call(call(ch,"getHighestNote"),"getAbsolutePitch"):null,"heads_center_y",box.y+box.height/2);
 }
 static Object inspect(Object measure)throws Exception {
  call(measure,"resetRhythm");Object sys=call(call(measure,"getStack"),"getSystem"),scale=call(call(sys,"getSheet"),"getScale"),sig=call(sys,"getSig");
  List<Object> all=new ArrayList<>((Collection<?>)call(measure,"getStandardChords"));Object rhythm=make("sheet.rhythm.MeasureRhythm",measure),retriever=make("sheet.rhythm.SlotsRetriever",measure,false);put(rhythm,"narrowSlotsRetriever",retriever);
  List<?> narrow=(List<?>)call(retriever,"buildSlots"),wide=(List<?>)call(make("sheet.rhythm.SlotsRetriever",measure,true),"buildSlots"),compounds=(List<?>)invoke(rhythm,"buildCompoundSlots",narrow,wide);((List)field(rhythm,"slots")).addAll(compounds);invoke(rhythm,"processStartingChords");
  List<Object> steps=new ArrayList<>();Class<?> mapperClass=c("sheet.rhythm.MeasureRhythm$SlotMapper");Constructor<?> ctor=mapperClass.getDeclaredConstructor(c("sheet.rhythm.MeasureRhythm"),c("sheet.rhythm.Slot$CompoundSlot"));ctor.setAccessible(true);
  for(Object slot:compounds) {
   Object mapper=ctor.newInstance(rhythm,slot);List<Object> times=new ArrayList<>();
   for(Object member:(List<?>)call(slot,"getMembers")){Map<?,?> t=(Map<?,?>)invoke(mapper,"readSlotTimes",member);for(var e:t.entrySet())times.add(obj("time",fraction(e.getKey()),"chord_ids",ids((Collection<?>)e.getValue(),all)));}
   invoke(mapper,"purgeRookiesSet");List<?> rookies=(List<?>)field(mapper,"rookies"),actives=(List<?>)invoke(mapper,"retrieveActives");Object mapping=null;List<Object> costs=new ArrayList<>();
   if(!rookies.isEmpty()&&!actives.isEmpty()) {
    Object vd=make("sheet.rhythm.VoiceDistance$Separated",scale);for(Object r:rookies)for(Object a:actives)costs.add(obj("rookie",all.indexOf(r),"active",all.indexOf(a),"left",distanceFact(a),"right",distanceFact(r),"distance",call(vd,"getDistance",a,r,(Object)null)));
    Object result=call(make("sheet.rhythm.ChordsMapper",rookies,actives,vd,field(mapper,"blackList"),field(mapper,"nextList")),"process");List<Object> pairs=new ArrayList<>();for(Object pair:(Collection<?>)field(result,"pairs"))pairs.add(obj("rookie",all.indexOf(field(pair,"rookie")),"active",all.indexOf(field(pair,"active"))));mapping=obj("cost",field(result,"cost"),"pairs",pairs);
   }
   List<Object> rookieIds=ids(rookies,all),activeIds=ids(actives,all);
   boolean ok=(Boolean)invoke(mapper,"mapChords");invoke(rhythm,"purgeExtinctVoices",slot);
   List<Object> after=new ArrayList<>();for(Object ch:(List<?>)call(slot,"getChords")){Object v=call(ch,"getVoice");after.add(obj("chord_id",all.indexOf(ch),"time_offset",fraction(call(ch,"getTimeOffset")),"voice",v==null?null:call(v,"getId")));}
   steps.add(obj("slot_id",call(slot,"getId"),"known_times",times,"rookies",rookieIds,"actives",activeIds,"costs",costs,"mapping",mapping,"ok",ok,"after",after));
  }
  return obj("number",call(call(measure,"getStack"),"getIdValue"),"interline",call(scale,"getInterline"),"implicit_tuplets",field(rhythm,"implicitTuplets"),"chord_bounds",all.stream().map(ch->{try{return rect(call(ch,"getBounds"));}catch(Exception e){throw new RuntimeException(e);}}).toList(),"steps",steps);
 }
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);List<Object> pages=new ArrayList<>();
  for(int i=1;i<args.length;i++){Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[i])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");List<Object> measures=new ArrayList<>();for(Object sys:(List<?>)call(sheet,"getSystems"))for(Object stack:(List<?>)call(sys,"getStacks"))measures.add(inspect(call(stack,"getFirstMeasure")));pages.add(obj("input",Path.of(args[i]).getFileName().toString(),"measures",measures));}
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[0]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","saved same-image original per-slot readSlotTimes/rookie/active/Separated distance/process mapping/apply; not general native polyphony or new image admission","pages",pages));System.exit(0);
 }
}
